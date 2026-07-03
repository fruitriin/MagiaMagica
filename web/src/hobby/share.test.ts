import { describe, expect, it } from "vite-plus/test";

import { buildShareUrl, decodeShare, encodeShare, parseShareHash } from "./share.ts";

describe("share encode/decode", () => {
  it("UTF-8 (日本語・記号込み) を round-trip する", () => {
    const src = `fn 召喚(x: i32) -> i32 {\n    x + 1 // コメント "引用"\n}`;
    expect(decodeShare(encodeShare(src))).toBe(src);
  });

  it("base64url なので URL 非安全文字 (+ /) を含まない", () => {
    // `+` `/` が出やすいバイト列を含むソース。
    const src = "fn f() { let _ = [0xff_u8, 0xfe, 0xfd, 0xfc]; }";
    const code = encodeShare(src);
    expect(code).not.toMatch(/[+/=]/);
    expect(decodeShare(code)).toBe(src);
  });

  it("壊れた UTF-8 (途中切断リンク) は例外になる — U+FFFD 置換で成功しない", () => {
    // "あ" (3バイト = base64url 4文字) を3文字に切ると多バイト列の途中で切れる。
    // SNS やチャットで URL が切り詰められたケースの再現。
    const truncated = encodeShare("あ").slice(0, 3);
    expect(() => decodeShare(truncated)).toThrow();
  });
});

describe("buildShareUrl / parseShareHash", () => {
  it("source と fn を載せて往復できる", () => {
    const url = buildShareUrl("https://x.test/", "fn g() {}", "g");
    const hash = url.slice(url.indexOf("#"));
    const parsed = parseShareHash(hash);
    expect(parsed.source).toBe("fn g() {}");
    expect(parsed.fn).toBe("g");
  });

  it("fn が null なら fn を載せない", () => {
    const url = buildShareUrl("https://x.test/", "fn g() {}", null);
    expect(parseShareHash(url.slice(url.indexOf("#"))).fn).toBeUndefined();
  });

  it("code が無いハッシュは空", () => {
    expect(parseShareHash("#foo=bar")).toEqual({});
    expect(parseShareHash("")).toEqual({});
  });

  it("壊れた code は source を載せずに返す (prefill フォールバック)", () => {
    // atob が投げる不正 base64。
    const parsed = parseShareHash("#code=!!!not-base64!!!");
    expect(parsed.source).toBeUndefined();
  });

  it("途中切断された code も全体を捨てて prefill フォールバックに倒れる", () => {
    const truncated = encodeShare("あ").slice(0, 3);
    expect(parseShareHash(`#code=${truncated}&fn=g`)).toEqual({});
  });

  it("style (ベルカ式) を載せて往復できる", () => {
    const url = buildShareUrl("https://x.test/", "fn g() {}", "g", "belka");
    const parsed = parseShareHash(url.slice(url.indexOf("#")));
    expect(parsed.style).toBe("belka");
    expect(parsed.fn).toBe("g");
  });

  it("style が null (既定のミッドチルダ式) なら載せない", () => {
    const url = buildShareUrl("https://x.test/", "fn g() {}", "g");
    expect(url).not.toContain("style=");
    expect(parseShareHash(url.slice(url.indexOf("#"))).style).toBeUndefined();
  });
});
