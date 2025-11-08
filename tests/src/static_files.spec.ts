import { test, expect } from "@playwright/test";

test.describe("Static Files Tests", () => {
  test("Serve CSS file", async ({ request }) => {
    const response = await request.get("/static/styles.css");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/css");

    const body = await response.text();
    expect(body).toContain("body {");
    expect(body).toContain("margin:");
    expect(body).toContain("}");
  });

  test("Serve JavaScript file", async ({ request }) => {
    const response = await request.get("/static/app.js");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/javascript");

    const body = await response.text();
    expect(body).toContain("function");
    expect(body).toContain("document");
  });

  test("Serve HTML file", async ({ request }) => {
    const response = await request.get("/static/index.html");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/html");

    const body = await response.text();
    expect(body).toContain("<!DOCTYPE html>");
    expect(body).toMatch(/<html[^>]*>/);
    expect(body).toContain("</html>");
  });

  test("Serve image file (PNG)", async ({ request }) => {
    const response = await request.get("/static/logo.png");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toBe("image/png");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
    expect(Buffer.isBuffer(buffer)).toBe(true);
  });

  test("Serve image file (JPEG)", async ({ request }) => {
    const response = await request.get("/static/photo.jpg");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toBe("image/jpeg");

    const buffer = await response.body();
    expect(buffer.length).toBeGreaterThan(0);
  });

  test("Serve JSON file", async ({ request }) => {
    const response = await request.get("/static/config.json");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");

    const body = await response.json();
    expect(typeof body).toBe("object");
    expect(body).toBeDefined();
  });

  test("Serve text file", async ({ request }) => {
    const response = await request.get("/static/readme.txt");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/plain");

    const body = await response.text();
    expect(body).toContain("This is a readme file");
  });

  test("File not found - 404", async ({ request }) => {
    const response = await request.get("/static/nonexistent.txt");

    expect(response.status()).toBe(404);
  });

  test("Directory traversal protection", async ({ request }) => {
    const response = await request.get("/static/../../../etc/passwd");

    expect(response.status()).toBe(404);
  });

  test("Serve file with query parameters", async ({ request }) => {
    const response = await request.get("/static/styles.css?v=1.0.0");

    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("text/css");

    const body = await response.text();
    expect(body).toContain("body {");
  });

  test("Last-Modified header", async ({ request }) => {
    const response = await request.get("/static/styles.css");

    expect(response.status()).toBe(200);

    const lastModified = response.headers()["last-modified"];
    expect(lastModified).toBeDefined();
    expect(new Date(lastModified!)).toBeInstanceOf(Date);
  });

  test("Conditional request - If-None-Match", async ({ request }) => {
    // First request to get ETag
    const firstResponse = await request.get("/static/styles.css");
    expect(firstResponse.status()).toBe(200);

    const etag = firstResponse.headers()["etag"];
    expect(etag).toBeDefined();

    // Second request with If-None-Match
    const secondResponse = await request.get("/static/styles.css", {
      headers: {
        "If-None-Match": etag ?? "",
      },
    });

    expect(secondResponse.status()).toBe(304);
    const body = await secondResponse.text();
    expect(body).toBe(""); // Empty body for 304
  });

  test("Conditional request - If-Modified-Since", async ({ request }) => {
    // First request to get Last-Modified
    const firstResponse = await request.get("/static/styles.css");
    expect(firstResponse.status()).toBe(200);

    const lastModified = firstResponse.headers()["last-modified"];
    expect(lastModified).toBeDefined();

    // Second request with If-Modified-Since
    const secondResponse = await request.get("/static/styles.css", {
      headers: {
        "If-Modified-Since": lastModified ?? "",
      },
    });

    expect(secondResponse.status()).toBe(304);
  });

  test("Multiple static routes", async ({ request }) => {
    const cssResponse = await request.get("/static/assets/styles.css");
    expect(cssResponse.status()).toBe(200);
    expect(cssResponse.headers()["content-type"]).toContain("text/css");

    const jsResponse = await request.get("/static/scripts/app.js");
    expect(jsResponse.status()).toBe(200);
    expect(jsResponse.headers()["content-type"]).toContain("text/javascript");
  });

  test("File extension without MIME type", async ({ request }) => {
    const response = await request.get("/static/unknown-file.asdxyz");

    expect(response.status()).toBe(200);
  });
});
