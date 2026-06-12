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
  /** IR 上のリング id。`SpellResponse.ring_excerpts` (ガード・ヘッダの断片) の
   *  引き当てに使う。IR 由来でないリング (凡例サンプル等) は null。 */
  irId: number | null;
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
  /** `SpellResponse.op_excerpts` の引き当てキー (`<ring_id>-<出現順>`)。
   *  IR 由来でない操作 (凡例サンプル等) は null。 */
  irKey: string | null;
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
  /** 呼び出し先の名前 (召喚印インスペクタの入力)。 */
  callTarget: string | null;
  /** IR 上の glyph id。`SpellResponse.call_excerpts` の引き当てに使う
   *  (セッション内の一時識別子 — 永続化しない)。 */
  irId: number;
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
  /** 接続元/先の Operation id。SVG 由来 (4.0.7) では null、IR 由来 (4.0.9) で埋まる。 */
  from: string | null;
  to: string | null;
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

/** 制御記号 (Phase 4.0.9 で意味論化 — 頂点計算は SymbolMark コンポーネントが行う)。 */
export type ControlSymbol = {
  id: string;
  kind: "branch" | "loop" | "early_return" | "return_branch" | "async_inner";
  /** アンカー位置 (branch/loop/async = リング中心、early_return/return_branch = 起点)。 */
  x: number;
  y: number;
  /** 対象リングの半径 (loop の軌道・early_return の長さ・async 内円に使う)。 */
  radius: number;
  /** early_return の向き (単位ベクトル)。他種別では [0,0]。 */
  direction: [number, number];
  /** 記号は必ずレイヤーに属する (control_flow か type_info — null は存在しない状態)。 */
  layer: SchemaLayer;
  z: number;
};

/** 意味論を復元しない要素の素通し (Phase 4.0.7 の過渡互換。4.0.9 のミッドチルダ
 *  IR 直結では使われず、常に空 — 型は将来の拡張余地として残す)。 */
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
  symbols: ControlSymbol[];
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

// ===== 配置済み IR (spec v0.3 §16、Phase 4.0.9) =====

/** 原文上の位置範囲 (1-based、end_column は exclusive — SpanIr の写し)。 */
export type IrSourceSpan = {
  start_line: number;
  end_line: number;
  start_column: number;
  end_column: number;
};

/** リング上の操作ドット (配置済み)。 */
export type IrOperation = {
  x: number;
  y: number;
  radius: number;
  effect: EffectCategory;
  /** 操作の原文位置。切り出し + ハイライトはサーバが行う (op_excerpts)。 */
  source_span: IrSourceSpan | null;
};

export type IrRing = {
  /** JSON 内の相互参照 (edges の from/to) 専用。永続化しない。 */
  id: number;
  role: "main" | "aux";
  x: number;
  y: number;
  radius: number;
  is_async: boolean;
  symbol: "branch" | "loop" | null;
  early_return: [number, number] | null;
  operations: IrOperation[];
  /** 補助リングのガード・ヘッダの原文位置 (`if cond` / `pat if guard` /
   *  `for pat in expr`)。メインリング・無条件の腕 (`else`) は null。 */
  guard_span: IrSourceSpan | null;
};

export type IrGlyph = {
  id: number;
  x: number;
  y: number;
  radius: number;
  effect: EffectCategory;
  /** 呼び出し先の名前。ピン可能判定はクライアントが関数一覧と照合する。 */
  call_target: string | null;
  /** 呼び出し式全体の原文位置。切り出し + ハイライトはサーバが行う
   *  (call_excerpts) — クライアントは未使用。 */
  source_span: IrSourceSpan | null;
};

export type IrEdge = {
  from: number;
  to: number;
};

/** 配置済み IR (ミッドチルダ式)。レイアウトは Rust 確定済み、Vue は描画専任。 */
export type IrSpell = {
  view_box: [number, number, number, number];
  rings: IrRing[];
  glyphs: IrGlyph[];
  edges: IrEdge[];
  signature: { text: string; arc_path: string } | null;
  return_branch: [number, number] | null;
};

// ===== 差分強調の配置済み IR (Phase 4.3 M4、spec v0.3 §8) =====

/** 差分強調マーク。配置 (中心・半径) は Rust 確定値 — removed は before 位置の
 *  本体半径 (ゴースト)、added/changed は after 位置のハロー半径。
 *  色・破線・線幅 (描き方) は MagicCircle の overlay 描画が持つ。
 *  配列順 = 描画順 (removed → changed → added、注目度順)。 */
export type DiffMark = {
  status: "added" | "changed" | "removed";
  x: number;
  y: number;
  radius: number;
};

// ===== ベルカ式の配置済み IR (Phase 4.3 M3、spec v0.3 §14) =====

/** 三極の語彙。色・ラベル文言は Vue 側 (BelkaCircle) が持つ。 */
export type BelkaPoleKind = "genesis" | "transmute" | "consume";

export type BelkaPoleIr = {
  pole: BelkaPoleKind;
  x: number;
  y: number;
  radius: number;
  field_radius: number;
  label_x: number;
  label_y: number;
  /** 操作ドット (phyllotaxis 配置済み)。 */
  dots: { x: number; y: number; effect: EffectCategory }[];
};

export type BelkaFlowIr = {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  width: number;
  /** 矢じりの頂点 (極円の縁)。羽は tip → 線端の方向から計算する。 */
  tip_x: number;
  tip_y: number;
};

/** ベルカ式の配置済み IR (データフロー三角力場)。 */
export type BelkaIr = {
  view_box: [number, number, number, number];
  poles: BelkaPoleIr[];
  flows: BelkaFlowIr[];
  signature: { text: string; x: number; y: number } | null;
};

/** ピン中心ビューの周辺チップ (Phase 4.1、配置は Rust 確定値)。 */
export type NeighborChip = {
  qualified: string;
  name: string;
  signature: string;
  /** リング距離: 1 = 同 impl、2 = 同ファイル (スタブ近接度 — 4.2 で本実装)。 */
  distance: number;
  x: number;
  y: number;
  scale: number;
  opacity: number;
  radius: number;
};

/** ピン中心ビューの全体配置 (`?with=neighbors` で併載)。 */
export type FocusLayout = {
  view_box: [number, number, number, number];
  neighbors: NeighborChip[];
};

/** `GET /spell/<fn>` のレスポンス。 */
export type SpellResponse = {
  qualified: string;
  signature: string;
  /** syntect でハイライト済みのソース HTML (サーバ生成、信頼済み入力)。 */
  source_html: string;
  start_line: number;
  /** 配置済み IR (ミッドチルダ式、spec v0.3 §16)。 */
  ir: IrSpell;
  /** 召喚印の呼び出し式 (glyph id → syntect ハイライト済み HTML)。
   *  レシーバ・引数込みの式全体を原文 (改行込み) から切り出した断片。 */
  call_excerpts: Record<string, string>;
  /** 操作ドットの原文断片 (`<ring_id>-<出現順>` = Operation.irKey →
   *  syntect ハイライト済み HTML)。ホバープレビュー用。 */
  op_excerpts: Record<string, string>;
  /** 補助リングのガード・ヘッダ断片 (ring id → syntect ハイライト済み HTML)。
   *  分岐の腕の条件・ループヘッダをリングホバーで見せる。 */
  ring_excerpts: Record<string, string>;
  /** ベルカ式の配置済み IR (Phase 4.3 — `<BelkaCircle>` が描く)。 */
  belka_ir: BelkaIr;
  /** スクリーンリーダー向けの呪文書き起こし (Phase 2.4)。 */
  transcript: string;
  /** ピン中心ビューの周辺配置 (`?with=neighbors` 時のみ)。 */
  focus_layout?: FocusLayout;
};
