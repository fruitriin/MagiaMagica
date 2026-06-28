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
});
