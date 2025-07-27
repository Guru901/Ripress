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
    // Should not serve files outside static directory
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
    expect(new Date(lastModified)).toBeInstanceOf(Date);
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
        "If-Modified-Since": lastModified,
      },
    });

    expect(secondResponse.status()).toBe(304);
  });

  //   test("Range request - partial content", async ({ request }) => {
  //     const response = await request.get("/static/large-file.txt", {
  //       headers: {
  //         Range: "bytes=0-99",
  //       },
  //     });

  //     expect(response.status()).toBe(206); // Partial Content
  //     expect(response.headers()["content-range"]).toContain("bytes 0-99");
  //     expect(response.headers()["accept-ranges"]).toBe("bytes");

  //     const body = await response.text();
  //     expect(body.length).toBeLessThanOrEqual(100);
  //   });

  //   test("Serve file from subdirectory", async ({ request }) => {
  //     const response = await request.get("/static/css/theme.css");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/css");

  //     const body = await response.text();
  //     expect(body).toContain(".theme");
  //   });

  //   test("Serve file from nested subdirectory", async ({ request }) => {
  //     const response = await request.get("/static/assets/images/icon.svg");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("image/svg+xml");

  //     const body = await response.text();
  //     expect(body).toContain("<svg");
  //     expect(body).toContain("</svg>");
  //   });

  //   test("Default index file", async ({ request }) => {
  //     const response = await request.get("/static/");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/html");

  //     const body = await response.text();
  //     expect(body).toContain("<!DOCTYPE html>");
  //   });

  //   test("Custom index file", async ({ request }) => {
  //     const response = await request.get("/static/docs/");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/html");

  //     const body = await response.text();
  //     expect(body).toContain("Documentation");
  //   });

  //   test("Font file serving", async ({ request }) => {
  //     const response = await request.get("/static/fonts/roboto.woff2");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toBe("font/woff2");

  //     const buffer = await response.body();
  //     expect(buffer.length).toBeGreaterThan(0);
  //   });

  //   test("PDF file serving", async ({ request }) => {
  //     const response = await request.get("/static/documents/manual.pdf");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toBe("application/pdf");

  //     const buffer = await response.body();
  //     expect(buffer.length).toBeGreaterThan(0);
  //   });

  //   test("Video file serving", async ({ request }) => {
  //     const response = await request.get("/static/videos/demo.mp4");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toBe("video/mp4");
  //     expect(response.headers()["accept-ranges"]).toBe("bytes");

  //     const buffer = await response.body();
  //     expect(buffer.length).toBeGreaterThan(0);
  //   });

  //   test("Static file with special characters in name", async ({ request }) => {
  //     const response = await request.get(
  //       "/static/files/document%20with%20spaces.txt"
  //     );

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/plain");

  //     const body = await response.text();
  //     expect(body).toContain("file with spaces");
  //   });

  //   test("Static file with non-ASCII characters", async ({ request }) => {
  //     const response = await request.get("/static/files/café-menu.txt");

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain("text/plain");

  //     const body = await response.text();
  //     expect(body).toContain("café");
  //   });

  //   test("Compression for static files", async ({ request }) => {
  //     const response = await request.get("/static/large-script.js", {
  //       headers: {
  //         "Accept-Encoding": "gzip, deflate",
  //       },
  //     });

  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toContain(
  //       "application/javascript"
  //     );

  //     // Check if compression is applied
  //     const contentEncoding = response.headers()["content-encoding"];
  //     if (contentEncoding) {
  //       expect(["gzip", "deflate"]).toContain(contentEncoding);
  //     }
  //   });

  //   test("CORS headers for static files", async ({ request }) => {
  //     const response = await request.get("/static/api-data.json", {
  //       headers: {
  //         Origin: "https://example.com",
  //       },
  //     });

  //     expect(response.status()).toBe(200);

  //     const corsHeader = response.headers()["access-control-allow-origin"];
  //     if (corsHeader) {
  //       expect(corsHeader).toBeDefined();
  //     }
  //   });

  //   test("Hidden files should not be served", async ({ request }) => {
  //     const response = await request.get("/static/.env");

  //     expect(response.status()).toBe(404);
  //     // Hidden files should not be accessible
  //   });

  //   test("Backup files should not be served", async ({ request }) => {
  //     const response = await request.get("/static/config.js.bak");

  //     expect(response.status()).toBe(404);
  //     // Backup files should not be accessible
  //   });

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
    expect(response.headers()["content-type"]).toContain(
      "application/octet-stream"
    );
  });
});
