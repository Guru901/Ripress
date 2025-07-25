import { test, expect } from "@playwright/test";

test.describe("Response Tests", () => {
  test("Get Cookie", async ({ request }) => {
    const cookieResponse = await request.get("/get-cookie-test");

    expect(cookieResponse.status()).toBe(200);

    const cookies = cookieResponse.headers()["set-cookie"];
    expect(cookies).toBeDefined();
    expect(cookies).toContain("test-cookie=value");
  });

  test("Set multiple cookies", async ({ request }) => {
    const response = await request.get("/multiple-cookies-test");

    expect(response.status()).toBe(200);

    const cookies = response.headers()["set-cookie"];
    expect(cookies).toBeDefined();
    expect(cookies).toContain("session=abc123");
    expect(cookies).toContain("theme=dark");
    expect(cookies).toContain("lang=en");
  });

  test("Set cookie with options", async ({ request }) => {
    const response = await request.get("/cookie-options-test");

    expect(response.status()).toBe(200);

    const cookies = response.headers()["set-cookie"];
    expect(cookies).toBeDefined();

    expect(cookies).toContain("secure-cookie=value");
    expect(cookies).toContain("HttpOnly");
    expect(cookies).toContain("Secure");
    expect(cookies).toContain("SameSite=Strict");
  });

  test("Set custom headers", async ({ request }) => {
    const response = await request.get("/custom-headers-test");

    expect(response.status()).toBe(200);

    const headers = response.headers();
    expect(headers["x-custom-header"]).toBe("custom-value");
    expect(headers["x-api-version"]).toBe("1.0");
    expect(headers["x-powered-by"]).toBe("Ripress");
  });

  test("Status code 201", async ({ request }) => {
    const response = await request.post("/created-test", {
      data: { name: "test" },
    });

    expect(response.status()).toBe(201);

    const body = await response.json();
    expect(body.created).toBe(true);
  });

  test("Custom status message", async ({ request }) => {
    const response = await request.get("/custom-status-test");

    expect(response.status()).toBe(418);
    expect(response.statusText()).toBe("I'm a teapot");
  });

  test("Redirect response", async ({ request }) => {
    const response = await request.get("/redirect-test", {
      maxRedirects: 0,
    });

    expect(response.status()).toBe(302);
    expect(response.headers()["location"]).toBe("/redirected");
  });

  test("Permanent redirect", async ({ request }) => {
    const response = await request.get("/permanent-redirect-test", {
      maxRedirects: 0,
    });

    expect(response.status()).toBe(301);
    expect(response.headers()["location"]).toBe("/new-location");
  });

  // test("Set vary header", async ({request}) => {
  //     const response = await request.get("/vary-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["vary"]).toBe("Accept-Encoding, User-Agent");
  // });
  //
  // test("JSON with status", async ({request}) => {
  //     const response = await request.get("/json-status-test");
  //
  //     expect(response.status()).toBe(422);
  //     expect(response.headers()["content-type"]).toContain("application/json");
  //
  //     const body = await response.json();
  //     expect(body.error).toBe("Validation failed");
  // });
  //
  // test("Send file attachment", async ({request}) => {
  //     const response = await request.get("/download-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-disposition"]).toContain('attachment; filename="test.txt"');
  //     expect(response.headers()["content-type"]).toContain("text/plain");
  // });
  //
  // test("Send file inline", async ({request}) => {
  //     const response = await request.get("/inline-file-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-disposition"]).toContain('inline; filename="document.pdf"');
  // });
  //
  // test("Set cache control", async ({request}) => {
  //     const response = await request.get("/cache-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["cache-control"]).toBe("public, max-age=3600");
  // });
  //
  // test("Set expires header", async ({request}) => {
  //     const response = await request.get("/expires-test");
  //
  //     expect(response.status()).toBe(200);
  //
  //     const expires = response.headers()["expires"];
  //     expect(expires).toBeDefined();
  //     expect(new Date(expires)).toBeInstanceOf(Date);
  // });
  //
  // test("Set etag header", async ({request}) => {
  //     const response = await request.get("/etag-test");
  //
  //     expect(response.status()).toBe(200);
  //
  //     const etag = response.headers()["etag"];
  //     expect(etag).toBeDefined();
  //     expect(etag).toMatch(/".*"/);
  // });
  //
  // test("Set last modified", async ({request}) => {
  //     const response = await request.get("/last-modified-test");
  //
  //     expect(response.status()).toBe(200);
  //
  //     const lastModified = response.headers()["last-modified"];
  //     expect(lastModified).toBeDefined();
  //     expect(new Date(lastModified)).toBeInstanceOf(Date);
  // });
  //
  // test("JSONP response", async ({request}) => {
  //     const response = await request.get("/jsonp-test?callback=myCallback");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/javascript");
  //
  //     const body = await response.text();
  //     expect(body).toMatch(/^myCallback\(/);
  //     expect(body).toContain('{"message":"success"}');
  // });
  //
  // test("Format response based on Accept header", async ({request}) => {
  //     const jsonResponse = await request.get("/format-test", {
  //         headers: {
  //             "Accept": "application/json"
  //         }
  //     });
  //
  //     expect(jsonResponse.status()).toBe(200);
  //     expect(jsonResponse.headers()["content-type"]).toContain("application/json");
  //
  //     const htmlResponse = await request.get("/format-test", {
  //         headers: {
  //             "Accept": "text/html"
  //         }
  //     });
  //
  //     expect(htmlResponse.status()).toBe(200);
  //     expect(htmlResponse.headers()["content-type"]).toContain("text/html");
  // });
  //
  // test("No content response", async ({request}) => {
  //     const response = await request.delete("/no-content-test");
  //
  //     expect(response.status()).toBe(204);
  //
  //     const body = await response.text();
  //     expect(body).toBe("");
  // });
  //
  // test("Set multiple headers at once", async ({request}) => {
  //     const response = await request.get("/multiple-headers-test");
  //
  //     expect(response.status()).toBe(200);
  //
  //     const headers = response.headers();
  //     expect(headers["x-header-1"]).toBe("value1");
  //     expect(headers["x-header-2"]).toBe("value2");
  //     expect(headers["x-header-3"]).toBe("value3");
  // });
  //
  // test("Remove header", async ({request}) => {
  //     const response = await request.get("/remove-header-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["x-removed-header"]).toBeUndefined();
  // });
  //
  // test("Set security headers", async ({request}) => {
  //     const response = await request.get("/security-headers-test");
  //
  //     expect(response.status()).toBe(200);
  //
  //     const headers = response.headers();
  //     expect(headers["x-frame-options"]).toBe("DENY");
  //     expect(headers["x-content-type-options"]).toBe("nosniff");
  //     expect(headers["x-xss-protection"]).toBe("1; mode=block");
  // });
  //
  // test("CORS headers", async ({request}) => {
  //     const response = await request.options("/cors-test", {
  //         headers: {
  //             "Origin": "https://example.com",
  //             "Access-Control-Request-Method": "POST"
  //         }
  //     });
  //
  //     expect(response.status()).toBe(200);
  //
  //     const headers = response.headers();
  //     expect(headers["access-control-allow-origin"]).toBe("https://example.com");
  //     expect(headers["access-control-allow-methods"]).toContain("POST");
  //     expect(headers["access-control-allow-headers"]).toBeDefined();
  // });
  //
  // test("Streaming response", async ({request}) => {
  //     const response = await request.get("/stream-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //
  //     const body = await response.text();
  //     expect(body).toContain("chunk1");
  //     expect(body).toContain("chunk2");
  // });
  //
  // test("Binary response", async ({request}) => {
  //     const response = await request.get("/binary-test");
  //
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toBe("application/octet-stream");
  //
  //     const buffer = await response.body();
  //     expect(buffer).toBeInstanceOf(Buffer);
  // });
});
