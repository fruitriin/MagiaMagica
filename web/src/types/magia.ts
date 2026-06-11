// MagiaMagica の境界スキーマとサーバ API 契約。
//
// `MagicCircleSchema` は Phase 4.0.7 (SVG パーサ) / 4.0.9 (IR ビルダ) が埋める
// 境界スキーマ。M2 (Phase 4.0.5) では型だけ先置きする (オーナー方針:
// 「スキーマだけ早く、ロジックは段階的」)。フィールドは意味論ベースで宣言し、
// レイアウトは Rust 任せ・Vue は描画専任 (POSD 分担) — 全要素が配置済 (x,y) を持つ。

/** 効果カテゴリ。palette.rs / filter::EffectCategory と同語彙 (spec §6.1.3)。 */
export type EffectCategory = "pure" | "io" | "network" | "db" | "filesystem" | "unsafe";

/** レンダリング様式 (spec v0.3 §14)。 */
export type RenderStyle = "midchilda" | "belka";

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
};

/** リング上の操作 (術式の1ステップ)。フォーカス・ピンの対象。 */
export type Operation = Placed & {
  id: string;
  /** 所属するリングの id。 */
  circleId: string;
  effect: EffectCategory;
  /** ピン/ホバーの対象になれるか (Phase 4.1 のピン中心ビューの入力)。 */
  selectable: boolean;
  /** ソース行へのリンク (Phase 4.0 ソース連動、Phase 4.4 呼び出しジャンプの入力)。 */
  line?: number;
};

/** データフローエッジ (Phase 3.4 Use-Def chains)。 */
export type Edge = {
  id: string;
  from: string;
  to: string;
  kind: "control" | "dataflow";
};

/** 効果記号 (召喚印など、操作の周囲に置かれる装飾的グリフ)。 */
export type EffectGlyph = Placed & {
  id: string;
  /** 記号を付随させる操作の id。 */
  operationId: string;
  effect: EffectCategory;
};

/** 関数のシグネチャ表示情報 (外周の textPath に相当)。 */
export type Signature = {
  name: string;
  qualified: string;
  /** `fn f(a: T) -> U` 形式の表示用文字列。 */
  text: string;
  implContext: string | null;
};

/**
 * 魔法陣1枚分の境界スキーマ。
 *
 * Rust (解析 + レイアウト) と Vue (描画 + インタラクション) の唯一の接点。
 * 4.0.7 は SVG パーサで、4.0.9 は IR JSON ビルダでこれを埋める。
 * Vue コンポーネント群 (`<MagicCircle :schema>`) は埋め方を知らない。
 */
export type MagicCircleSchema = {
  signature: Signature;
  style: RenderStyle;
  /** SVG viewBox: [minX, minY, width, height]。 */
  viewBox: [number, number, number, number];
  circles: Circle[];
  operations: Operation[];
  edges: Edge[];
  glyphs: EffectGlyph[];
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

/** `GET /state` のレスポンス。 */
export type StateResponse = {
  /** 構文エラー中は last-good を返しつつここにメッセージが入る。 */
  error: string | null;
  file: string;
  functions: FunctionMeta[];
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
