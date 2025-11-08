import { test, expect } from "@playwright/test";

test.describe("Response Streaming Tests", () => {
  // test("Stream text data to client", async ({ request }) => {
  //   const response = await request.get("/stream-text");
  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //   expect(response.headers()["content-type"]).toContain("text/event-stream");
  //   const body = await response.text();
  //   expect(body).toContain("chunk1");
  //   expect(body).toContain("chunk2");
  //   expect(body).toContain("chunk3");
  // });
  // test("Stream JSON data to client", async ({ request }) => {
  //   const response = await request.get("/stream-json");
  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //   expect(response.headers()["content-type"]).toContain("text/event-stream");
  //   const body = await response.text();
  //   expect(body).toContain('{"id":1');
  //   expect(body).toContain('{"id":2');
  //   expect(body).toContain('{"id":3');
  // });
  //   test("Stream large file to client", async ({ request }) => {
  //     const response = await request.get("/stream-large-file");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("text/plain");
  //     const body = await response.text();
  //     expect(body.length).toBeGreaterThan(10000); // Large file
  //     expect(body).toContain("file content");
  //   });
  //   test("Stream CSV data to client", async ({ request }) => {
  //     const response = await request.get("/stream-csv");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("text/csv");
  //     const body = await response.text();
  //     expect(body).toContain("name,email,age");
  //     expect(body).toContain("user1@example.com");
  //     expect(body).toContain("user2@example.com");
  //   });
  //   test("Stream XML data to client", async ({ request }) => {
  //     const response = await request.get("/stream-xml");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("application/xml");
  //     const body = await response.text();
  //     expect(body).toContain("<?xml version");
  //     expect(body).toContain("<users>");
  //     expect(body).toContain("</users>");
  //     expect(body).toContain("<user id=");
  //   });
  //   test("Stream with Server-Sent Events", async ({ request }) => {
  //     const response = await request.get("/stream-sse");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-type"]).toBe("text/event-stream");
  //     expect(response.headers()["cache-control"]).toBe("no-cache");
  //     expect(response.headers()["connection"]).toBe("keep-alive");
  //     const body = await response.text();
  //     expect(body).toContain("data: ");
  //     expect(body).toContain("event: ");
  //     expect(body).toContain("\n\n");
  //   });
  //   test("Stream binary data to client", async ({ request }) => {
  //     const response = await request.get("/stream-binary");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toBe("application/octet-stream");
  //     const buffer = await response.body();
  //     expect(buffer.length).toBeGreaterThan(0);
  //     expect(Buffer.isBuffer(buffer)).toBe(true);
  //   });
  //   test("Stream with custom headers", async ({ request }) => {
  //     const response = await request.get("/stream-custom-headers");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["x-stream-id"]).toBeDefined();
  //     expect(response.headers()["x-chunk-size"]).toBeDefined();
  //     const body = await response.text();
  //     expect(body).toContain("streamed content");
  //   });
  //   test("Stream with compression", async ({ request }) => {
  //     const response = await request.get("/stream-compressed", {
  //       headers: {
  //         "Accept-Encoding": "gzip",
  //       },
  //     });
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["content-encoding"]).toBe("gzip");
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     const body = await response.text();
  //     expect(body).toContain("compressed streamed data");
  //   });
  //   test("Stream with slow data transmission", async ({ request }) => {
  //     const startTime = Date.now();
  //     const response = await request.get("/stream-slow");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     const endTime = Date.now();
  //     const body = await response.text();
  //     expect(body).toContain("slow chunk 1");
  //     expect(body).toContain("slow chunk 2");
  //     expect(endTime - startTime).toBeGreaterThan(1000); // Should take at least 1 second
  //   });
  //   test("Stream template rendering", async ({ request }) => {
  //     const response = await request.get("/stream-template");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("text/html");
  //     const body = await response.text();
  //     expect(body).toContain("<html>");
  //     expect(body).toContain("<head>");
  //     expect(body).toContain("<body>");
  //     expect(body).toContain("</html>");
  //   });
  //   test("Stream API data pagination", async ({ request }) => {
  //     const response = await request.get("/stream-paginated-api");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("application/json");
  //     const body = await response.text();
  //     expect(body).toContain('"page":1');
  //     expect(body).toContain('"page":2');
  //     expect(body).toContain('"page":3');
  //     expect(body).toContain('"hasMore"');
  //   });
  //   test("Stream database query results", async ({ request }) => {
  //     const response = await request.get("/stream-db-results");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("application/json");
  //     const body = await response.text();
  //     expect(body).toContain('"record_id"');
  //     expect(body).toContain('"timestamp"');
  //     expect(body).toMatch(/\{"record_id":\d+/g); // Multiple records
  //   });
  //   test("Stream with error in middle", async ({ request }) => {
  //     const response = await request.get("/stream-with-error");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     const body = await response.text();
  //     expect(body).toContain("chunk before error");
  //     // Note: The connection might be closed after error, so we just check initial content
  //   });
  //   test("Stream log files", async ({ request }) => {
  //     const response = await request.get("/stream-logs");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("text/plain");
  //     const body = await response.text();
  //     expect(body).toContain("[INFO]");
  //     expect(body).toContain("[ERROR]");
  //     expect(body).toContain("[DEBUG]");
  //     expect(body).toMatch(/\d{4}-\d{2}-\d{2}/); // Date format
  //   });
  //   test("Stream with content-length and chunked", async ({ request }) => {
  //     const response = await request.get("/stream-mixed-encoding");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     // Content-Length should not be present with chunked encoding
  //     const body = await response.text();
  //     expect(body).toContain("mixed encoding content");
  //   });
  //   test("Stream NDJSON (Newline Delimited JSON)", async ({ request }) => {
  //     const response = await request.get("/stream-ndjson");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain(
  //       "application/x-ndjson"
  //     );
  //     const body = await response.text();
  //     const lines = body.trim().split("\n");
  //     expect(lines.length).toBeGreaterThan(1);
  //     // Each line should be valid JSON
  //     lines.forEach((line) => {
  //       if (line.trim()) {
  //         expect(() => JSON.parse(line)).not.toThrow();
  //       }
  //     });
  //   });
  //   test("Stream with custom chunk sizes", async ({ request }) => {
  //     const response = await request.get("/stream-custom-chunks");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["x-chunk-strategy"]).toBe("custom");
  //     const body = await response.text();
  //     expect(body).toContain("small chunk");
  //     expect(body).toContain("medium chunk");
  //     expect(body).toContain("large chunk");
  //   });
  //   test("Stream real-time data feed", async ({ request }) => {
  //     const response = await request.get("/stream-realtime-feed");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-type"]).toContain("application/json");
  //     expect(response.headers()["x-realtime"]).toBe("true");
  //     const body = await response.text();
  //     expect(body).toContain('"timestamp"');
  //     expect(body).toContain('"data"');
  //     expect(body).toContain('"sequence"');
  //   });
  //   test("Stream file download with progress", async ({ request }) => {
  //     const response = await request.get("/stream-download-progress");
  //     expect(response.status()).toBe(200);
  //     expect(response.headers()["transfer-encoding"]).toBe("chunked");
  //     expect(response.headers()["content-disposition"]).toContain("attachment");
  //     expect(response.headers()["x-total-size"]).toBeDefined();
  //     const body = await response.text();
  //     expect(body.length).toBeGreaterThan(1000);
  //   });
});
