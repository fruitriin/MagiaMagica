// MagiaMagica の境界スキーマとサーバ API 契約。
//
// `MagicCircleSchema` は Rust (解析 + レイアウト) と Vue (描画 + インタラクション) の
// 唯一の接点 (Phase 4.0.7 で確立、spec §5.4 / §6.1)。
// Phase 4.0.7 では SVG パーサ (converters/svgToSchema.ts) が埋め、
// Phase 4.0.9 で IR JSON ビルダに差し替える — Vue コンポーネント群は無修正で流用する。
// レイアウトは Rust 任せ・Vue は描画専任 (POSD 分担) — 全要素が配置済み座標を持つ。

/** 効果カテゴリ。palette.rs / filter::EffectCategory と同語彙 (spec §6.1.3)。 */
export type EffectCategory = "pure" | "io" | "network" | "db" | "filesystem" | "unsafe";

/** レンダリング様式 (spec v0.3 §14)。 */
export type RenderStyle = "midchilda" | "belka";

/** レイヤー名は Rust 側 FilterSpec / SVG の g.layer-* と同語彙 (spec §8)。 */
export type SchemaLayer = "control_flow" | "effects" | "type_info";

/** 配置済み座標。SVG viewBox 座標系 (y 下向き)。 */
export type Placed = {
  x: number;
  y: number;
};

/** 中央魔法陣または補助リング。 */
export type Circle = Placed & {
  id: string;
  /** 中央陣 = main、召喚陣などの補助リング = aux。 */
  role: "main" | "aux";
  radius: number;
  strokeWidth: number;
  layer: SchemaLayer | null;
  /** 描画順 (SVG の出現順)。重なりの z-order を元レンダラと一致させる。 */
  z: number;
};

/** リング上の操作 (術式の1ステップ)。ホバー・選択・ピンの対象。 */
export type Operation = Placed & {
  id: string;
  radius: number;
  /** 効果カテゴリの色 (palette.rs の16進値)。描画はこの値をそのまま使う (画素等価)。 */
  color: string;
  /** 色から逆引きした効果カテゴリ (palette 語彙に無い色なら null)。4.1+ の意味論用。 */
  effect: EffectCategory | null;
  /** ホバー/選択の対象になれるか (Phase 4.1 のピン中心ビューの入力)。 */
  selectable: boolean;
  layer: SchemaLayer | null;
  z: number;
};

/** 召喚印 (外部呼び出しを示す大きめの円、効果カテゴリ色)。
 *  Phase 4.4 (呼び出しジャンプ) でクリック対象になるため selectable を持つ。 */
export type EffectGlyph = Placed & {
  id: string;
  radius: number;
  color: string;
  effect: EffectCategory | null;
  selectable: boolean;
  layer: SchemaLayer | null;
  z: number;
};

/** 制御フローの接続線。Phase 4.0.7 (SVG 由来) は座標のみ。
 *  from/to の id 参照は Phase 4.0.9 (IR 由来) で埋まる。 */
export type SchemaEdge = {
  id: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  layer: SchemaLayer | null;
  z: number;
  from?: string;
  to?: string;
};

/** 関数のシグネチャ表示。ミッドチルダ式は外周の円弧 textPath、ベルカ式は直線配置。 */
export type Signature = {
  text: string;
  fontSize: number;
  /** 円弧の path データ (defs の sig-arc)。直線配置 (ベルカ式) では null。 */
  arcPath: string | null;
  /** 直線配置の座標 (arcPath が null のとき使用)。 */
  x: number;
  y: number;
};

/** 意味論を復元しない要素の素通し (複合記号 sym-*、ベルカ式の力場など)。
 *  markup はサーバ生成 SVG の断片 (信頼済み入力) で、表示維持のためそのまま描画する。 */
export type RawElement = {
  id: string;
  layer: SchemaLayer | null;
  markup: string;
  z: number;
};

/**
 * 魔法陣1枚分の境界スキーマ。
 *
 * id は「SVG 内の出現順」によるセッション内の一時識別子であり、IR の SigilId とは
 * 無関係 (SigilId は外部契約に出さない — Phase 3.2 の情報隠蔽方針)。
 * 永続参照 (URL 等) には使わないこと。
 */
export type MagicCircleSchema = {
  signature: Signature | null;
  style: RenderStyle;
  /** SVG viewBox: [minX, minY, width, height]。 */
  viewBox: [number, number, number, number];
  circles: Circle[];
  operations: Operation[];
  edges: SchemaEdge[];
  glyphs: EffectGlyph[];
  raws: RawElement[];
};

// ===== サーバ API 契約 (magia serve, Phase 4.0) =====

/** `/state` の関数一覧エントリ。 */
export type FunctionMeta = {
  name: string;
  qualified: string;
  signature: string;
  start_line: number;
  end_line: number;
  impl_context: string | null;
};

/** 構文エラーの内容。エラー中もサーバは last-good スナップショットを配信し続ける。 */
export type ServerError = {
  message: string;
  /** エラー行 (1-origin)。位置を特定できないエラーでは null。 */
  line: number | null;
};

/** `GET /state` のレスポンス。 */
export type StateResponse = {
  /** 構文エラー中は last-good を返しつつここに内容が入る。 */
  error: ServerError | null;
  file: string;
  functions: FunctionMeta[];
  /** ファイル更新ごとに進む世代番号 (SSE の `data:` と同じ系列)。 */
  version: number;
};

/** `GET /spell/<fn>` のレスポンス。 */
export type SpellResponse = {
  qualified: string;
  signature: string;
  /** syntect でハイライト済みのソース HTML (サーバ生成、信頼済み入力)。 */
  source_html: string;
  start_line: number;
  svg: string;
  svg_belka: string;
  /** スクリーンリーダー向けの呪文書き起こし (Phase 2.4)。 */
  transcript: string;
};
