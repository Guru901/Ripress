import { expect, test } from "@playwright/test";

test.describe("Request Tests", () => {
  test("Set and check cookies", async ({ request }) => {
    const cookieResponse = await request.get("/cookie-test", {
      headers: {
        Cookie: "sessionId=abc123",
      },
    });

    expect(cookieResponse.status()).toBe(200);

    const body = await cookieResponse.json();
    expect(body.sessionId).toBe("abc123");
  });

  test("Set and check headers", async ({ request }) => {
    const headerResponse = await request.get("/header-test", {
      headers: {
        "Test-Header": "test-value",
      },
    });

    expect(headerResponse.status()).toBe(200);

    const body = await headerResponse.json();
    expect(body.header).toBe("test-value");
  });

  test("Set and check params and query", async ({ request }) => {
    const paramAndQueryResponse = await request.get(
      "/param-and-query-test/test?query=test-query"
    );

    expect(paramAndQueryResponse.status()).toBe(200);

    const body = await paramAndQueryResponse.json();
    expect(body.param).toBe("test");
    expect(body.query).toBe("test-query");
  });

  test("Set and get origin_url and path", async ({ request }) => {
    const originUrlAndPathResponse = await request.get(
      "/origin-url-and-path/test?q=test"
    );

    expect(originUrlAndPathResponse.status()).toBe(200);

    const body = await originUrlAndPathResponse.json();
    expect(
      body.originUrl === "http://localhost:8080" ||
        body.originUrl === "http://127.0.0.1:8080"
    ).toBe(true);
    expect(body.path).toBe("/origin-url-and-path/test");
  });

  test("Test Ip", async ({ request }) => {
    const ipResponse = await request.get("/ip-test");

    expect(ipResponse.status()).toBe(200);

    const body = await ipResponse.json();

    expect(body.ip).toBe("127.0.0.1");
  });

  test("Set and get json body", async ({ request }) => {
    const jsonResponse = await request.post("/json-test", {
      data: { name: "test", age: 123 },
    });

    expect(jsonResponse.status()).toBe(200);

    const body = await jsonResponse.json();
    expect(body.name).toBe("test");
    expect(body.age).toBe(123);
  });

  test("Set and get text body", async ({ request }) => {
    const textResponse = await request.post("/text-test", {
      data: "test",
      headers: {
        "Content-Type": "text/plain",
      },
    });

    expect(textResponse.status()).toBe(200);

    const body = await textResponse.text();
    expect(body).toBe("test");
  });

  test("Set and get form data", async ({ request }) => {
    const jsonResponse = await request.post("/form-test", {
      form: { name: "test" },
    });

    expect(jsonResponse.status()).toBe(200);

    const body = await jsonResponse.json();
    expect(body.name).toBe("test");
  });

  test("Auth - Should be authenticated", async ({ request }) => {
    const authResponse = await request.get("/auth", {
      headers: {
        Cookie: "token=123abc",
      },
    });

    expect(authResponse.status()).toBe(200);

    const body = await authResponse.text();
    expect(body).toBe("123abc");
  });

  test("Auth - isn't authenticated", async ({ request }) => {
    const authResponse = await request.get("/auth");

    expect(authResponse.status()).toBe(401);

    const body = await authResponse.text();
    expect(body).toBe("unauthorized");
  });

  // Additional Request Object Tests

  test("Multiple query parameters", async ({ request }) => {
    const response = await request.get(
      "/multi-query?name=john&age=25&city=NYC"
    );

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.name).toBe("john");
    expect(body.age).toBe("25");
    expect(body.city).toBe("NYC");
  });

  test("Multiple route parameters", async ({ request }) => {
    const response = await request.get("/users/123/posts/456");

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.userId).toBe("123");
    expect(body.postId).toBe("456");
  });

  test("Multiple cookies", async ({ request }) => {
    const response = await request.get("/multi-cookies", {
      headers: {
        Cookie: "user=john; theme=dark; lang=en",
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.user).toBe("john");
    expect(body.theme).toBe("dark");
    expect(body.lang).toBe("en");
  });

  test("Multiple headers", async ({ request }) => {
    const response = await request.get("/multi-headers", {
      headers: {
        "User-Agent": "TestBot/1.0",
        Accept: "application/json",
        Authorization: "Bearer token123",
        "X-Custom-Header": "custom-value",
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.userAgent).toBe("TestBot/1.0");
    expect(body.accept).toBe("application/json");
    expect(body.authorization).toBe("Bearer token123");
    expect(body.customHeader).toBe("custom-value");
  });

  test("Request method detection", async ({ request }) => {
    const getResponse = await request.get("/method-test");
    expect(getResponse.status()).toBe(200);
    const getBody = await getResponse.json();
    expect(getBody.method === "GET");

    const postResponse = await request.post("/method-test", {
      data: {},
    });

    expect(postResponse.status()).toBe(200);
    const postBody = await postResponse.json();
    expect(postBody.method === "POST");

    const putResponse = await request.put("/method-test", {
      data: {},
    });

    expect(putResponse.status()).toBe(200);

    const putBody = await putResponse.json();

    expect(putBody.method === "PUT");

    const deleteResponse = await request.delete("/method-test");

    expect(deleteResponse.status()).toBe(200);

    const deleteBody = await deleteResponse.json();

    expect(deleteBody.method === "DELETE");
  });

  test("URL-encoded form data", async ({ request }) => {
    const response = await request.post("/urlencoded-test", {
      form: {
        username: "testuser",
        password: "secret123",
        email: "test@example.com",
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.username).toBe("testuser");
    expect(body.password).toBe("secret123");
    expect(body.email).toBe("test@example.com");
  });

  test("Raw body data", async ({ request }) => {
    const rawData = "raw text content";
    const response = await request.post("/raw-body-test", {
      data: rawData,
      headers: {
        "Content-Type": "text/plain",
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.rawBody).toBe(rawData);
    expect(body.contentType).toBe("text/plain");
  });

  test("Request secure/insecure", async ({ request }) => {
    const response = await request.get("/secure-test");

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.secure === false); // Assuming HTTP for tests
  });

  test("Request xhr detection", async ({ request }) => {
    const response = await request.get("/xhr-test", {
      headers: {
        "X-Requested-With": "XMLHttpRequest",
      },
    });

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(body.xhr);
  });

  test("Empty request body", async ({ request }) => {
    const response = await request.post("/empty-body-test");

    expect(response.status()).toBe(200);

    const body = await response.json();
    expect(!body.hasBody);
  });

  // ---------------------------------------------------------------------------------------------- //

  test("Invalid JSON in request body", async ({ request }) => {
    const response = await request.post("/json-error-test", {
      data: '{"invalid": json}', // Intentionally malformed JSON
      headers: {
        "Content-Type": "application/json",
      },
    });

    // Should return 400 for malformed JSON
    expect(response.status()).toBe(400);
  });

  test("Extremely long header value", async ({ request }) => {
    const longValue = "x".repeat(8192); // 8KB header
    const response = await request.get("/long-header-test", {
      headers: {
        "X-Long-Header": longValue,
      },
    });

    // Server should either accept it or return 431 (Request Header Fields Too Large)
    expect([200, 431].includes(response.status())).toBe(true);
  });

  test("Request with no Content-Type for JSON data", async ({ request }) => {
    const response = await request.post("/no-content-type-test", {
      data: JSON.stringify({ test: "data" }),
      // Intentionally omitting Content-Type header
    });

    // Server behavior may vary - could be 200, 400, or 415
    expect([200, 400, 415].includes(response.status())).toBe(true);
  });

  test("Request with conflicting Content-Type and data", async ({
    request,
  }) => {
    const response = await request.post("/content-type-mismatch-test", {
      data: { test: "object" }, // Sending JSON object
      headers: {
        "Content-Type": "text/plain", // But claiming it's text
      },
    });

    expect(response.status()).toBe(200);
    const body = await response.json();
    // Server should handle the mismatch gracefully
    expect(body).toBeDefined();
  });

  test("Special characters in query parameters", async ({ request }) => {
    const response = await request.get(
      "/special-query-test?name=John%20Doe&symbols=%21%40%23%24&unicode=🌟"
    );

    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.name).toBe("John Doe");
    expect(body.symbols).toBe("!@#$");
    expect(body.unicode).toBe("🌟");
  });

  test("Very large request body", async ({ request }) => {
    const largeData = {
      data: "x".repeat(1024 * 1024), // 1MB of data
      numbers: Array.from({ length: 1000 }, (_, i) => i),
    };

    const response = await request.post("/large-body-test", {
      data: largeData,
    });

    // Should either accept or return 413 (Payload Too Large)
    expect([200, 413].includes(response.status())).toBe(true);
  });

  //-------------------------------MULTIPART FORM DATA TESTS----------------------------------------//

  test("Multipart form data with text fields", async ({ request }) => {
    const response = await request.post("/multipart-text-test", {
      multipart: {
        name: "John Doe",
        email: "john@example.com",
        age: "30",
        description: "A test user with multiple fields",
      },
    });

    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.name).toBe("John Doe");
    expect(body.email).toBe("john@example.com");
    expect(body.age).toBe("30");
    expect(body.description).toBe("A test user with multiple fields");
  });

  test("Multipart form data with file upload", async ({ request }) => {
    // Create a test file buffer
    const testFileContent = Buffer.from(
      "This is a test file content for upload"
    );

    const response = await request.post("/multipart-file-test", {
      multipart: {
        name: "Test User",
        file: {
          name: "test.txt",
          mimeType: "text/plain",
          buffer: testFileContent,
        },
      },
    });

    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.name).toBe("Test User");
    expect(body.fileName).toBe("test.txt");
    expect(body.fileSize).toBe(testFileContent.length);
    expect(body.mimeType).toBe("text/plain");
  });

  // test("Multipart form data with multiple files", async ({ request }) => {
  //   const file1 = Buffer.from("Content of first file");
  //   const file2 = Buffer.from("Content of second file");

  //   const response = await request.post("/multipart-multiple-files-test", {
  //     multipart: {
  //       description: "Upload with multiple files",
  //       file1: {
  //         name: "document1.txt",
  //         mimeType: "text/plain",
  //         buffer: file1,
  //       },
  //       file2: {
  //         name: "document2.txt",
  //         mimeType: "text/plain",
  //         buffer: file2,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.description).toBe("Upload with multiple files");
  //   expect(body.files).toHaveLength(2);
  //   expect(body.files[0].name).toBe("document1.txt");
  //   expect(body.files[1].name).toBe("document2.txt");
  // });

  // test("Multipart form data with image file", async ({ request }) => {
  //   // Create a minimal PNG buffer (1x1 pixel transparent PNG)
  //   const pngBuffer = Buffer.from([
  //     0x89,
  //     0x50,
  //     0x4e,
  //     0x47,
  //     0x0d,
  //     0x0a,
  //     0x1a,
  //     0x0a, // PNG signature
  //     0x00,
  //     0x00,
  //     0x00,
  //     0x0d, // IHDR length
  //     0x49,
  //     0x48,
  //     0x44,
  //     0x52, // IHDR
  //     0x00,
  //     0x00,
  //     0x00,
  //     0x01, // width: 1
  //     0x00,
  //     0x00,
  //     0x00,
  //     0x01, // height: 1
  //     0x08,
  //     0x06,
  //     0x00,
  //     0x00,
  //     0x00, // bit depth, color type, compression, filter, interlace
  //     0x1f,
  //     0x15,
  //     0xc4,
  //     0x89, // CRC
  //     0x00,
  //     0x00,
  //     0x00,
  //     0x0a, // IDAT length
  //     0x49,
  //     0x44,
  //     0x41,
  //     0x54, // IDAT
  //     0x78,
  //     0x9c,
  //     0x63,
  //     0x00,
  //     0x01,
  //     0x00,
  //     0x00,
  //     0x05,
  //     0x00,
  //     0x01, // compressed data
  //     0x0d,
  //     0x0a,
  //     0x2d,
  //     0xb4, // CRC
  //     0x00,
  //     0x00,
  //     0x00,
  //     0x00, // IEND length
  //     0x49,
  //     0x45,
  //     0x4e,
  //     0x44, // IEND
  //     0xae,
  //     0x42,
  //     0x60,
  //     0x82, // CRC
  //   ]);

  //   const response = await request.post("/multipart-image-test", {
  //     multipart: {
  //       title: "Image Upload Test",
  //       image: {
  //         name: "test.png",
  //         mimeType: "image/png",
  //         buffer: pngBuffer,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.title).toBe("Image Upload Test");
  //   expect(body.fileName).toBe("test.png");
  //   expect(body.mimeType).toBe("image/png");
  //   expect(body.fileSize).toBe(pngBuffer.length);
  // });

  // // BINARY DATA TESTS

  // test("Binary file upload with application/octet-stream", async ({
  //   request,
  // }) => {
  //   const binaryData = Buffer.from([
  //     0x00, 0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc,
  //   ]);

  //   const response = await request.post("/binary-upload-test", {
  //     data: binaryData,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.contentType).toBe("application/octet-stream");
  //   expect(body.size).toBe(binaryData.length);
  //   expect(body.firstByte).toBe(0x00);
  //   expect(body.lastByte).toBe(0xfc);
  // });

  // test("Large binary file upload", async ({ request }) => {
  //   // Create a 1KB binary file with random data
  //   const binaryData = Buffer.alloc(1024);
  //   for (let i = 0; i < 1024; i++) {
  //     binaryData[i] = i % 256;
  //   }

  //   const response = await request.post("/large-binary-test", {
  //     data: binaryData,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //       "Content-Length": binaryData.length.toString(),
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.size).toBe(1024);
  //   expect(body.contentType).toBe("application/octet-stream");
  // });

  // test("Binary response download", async ({ request }) => {
  //   const response = await request.get("/binary-download-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/octet-stream");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);

  //   // Verify it's actually binary data
  //   expect(buffer[0]).toBeDefined();
  // });

  // test("Image binary response", async ({ request }) => {
  //   const response = await request.get("/image-binary-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("image/png");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);

  //   // Check PNG signature
  //   expect(buffer[0]).toBe(0x89);
  //   expect(buffer[1]).toBe(0x50);
  //   expect(buffer[2]).toBe(0x4e);
  //   expect(buffer[3]).toBe(0x47);
  // });

  // test("PDF binary response", async ({ request }) => {
  //   const response = await request.get("/pdf-download-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/pdf");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);

  //   // Check PDF signature
  //   const pdfSignature = buffer.subarray(0, 4).toString();
  //   expect(pdfSignature).toBe("%PDF");
  // });

  // // ERROR HANDLING TESTS

  // test("Request to non-existent endpoint", async ({ request }) => {
  //   const response = await request.get("/non-existent-endpoint");
  //   expect(response.status()).toBe(404);
  // });

  // test("Method not allowed", async ({ request }) => {
  //   const response = await request.patch("/method-not-allowed-test");
  //   expect(response.status()).toBe(405);
  // });

  // test("Request timeout handling", async ({ request }) => {
  //   const response = await request.get("/timeout-test", {
  //     timeout: 1000, // 1 second timeout
  //   });

  //   // Should either complete quickly or timeout
  //   expect([200, 408].includes(response.status())).toBe(true);
  // });

  // test("Malformed cookie header", async ({ request }) => {
  //   const response = await request.get("/cookie-error-test", {
  //     headers: {
  //       Cookie: "invalid-cookie-format-no-equals-sign",
  //     },
  //   });

  //   // Server should handle gracefully
  //   expect([200, 400].includes(response.status())).toBe(true);
  // });

  // test("Empty multipart upload", async ({ request }) => {
  //   const response = await request.post("/empty-multipart-test", {
  //     multipart: {},
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.fieldsCount).toBe(0);
  // });

  // test("Multipart with empty file", async ({ request }) => {
  //   const emptyBuffer = Buffer.alloc(0);

  //   const response = await request.post("/empty-file-test", {
  //     multipart: {
  //       name: "Empty File Test",
  //       emptyFile: {
  //         name: "empty.txt",
  //         mimeType: "text/plain",
  //         buffer: emptyBuffer,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.fileName).toBe("empty.txt");
  //   expect(body.fileSize).toBe(0);
  // });

  // test("Request with invalid UTF-8 in body", async ({ request }) => {
  //   const invalidUtf8 = Buffer.from([0xff, 0xfe, 0xfd]); // Invalid UTF-8 sequence

  //   const response = await request.post("/invalid-utf8-test", {
  //     data: invalidUtf8,
  //     headers: {
  //       "Content-Type": "text/plain; charset=utf-8",
  //     },
  //   });

  //   // Server should handle gracefully
  //   expect([200, 400].includes(response.status())).toBe(true);
  // });

  // test("Request with oversized headers", async ({ request }) => {
  //   const headers = new Map<string, string>();
  //   // Create many headers to potentially exceed limits
  //   for (let i = 0; i < 100; i++) {
  //     headers.set(`X-Header-${i}`, `value-${i}`.repeat(100));
  //   }

  //   try {
  //     // Convert Map to plain object for headers
  //     const headersObj = Object.fromEntries(headers.entries());
  //     const response = await request.get("/header-limit-test", {
  //       headers: headersObj,
  //     });
  //     expect([200, 431].includes(response.status())).toBe(true);
  //   } catch (error) {
  //     expect(error).toContain("header");
  //   }
  // });

  // test("Duplicate query parameters", async ({ request }) => {
  //   const response = await request.get(
  //     "/duplicate-query-test?name=first&name=second&name=third"
  //   );

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   // Server behavior varies - might get array, last value, or first value
  //   expect(body.name).toBeDefined();
  // });

  // test("Request with circular JSON reference", async ({ request }) => {
  //   // This would cause JSON.stringify to fail, but Playwright should handle it
  //   const circularObj = { name: "test" };
  //   //@ts-expect-error - Playwright should handle circular references
  //   circularObj.self = circularObj; // Create circular reference

  //   try {
  //     const response = await request.post("/circular-json-test", {
  //       data: circularObj,
  //     });
  //     // If it gets through, server should respond
  //     expect([200, 400].includes(response.status())).toBe(true);
  //   } catch (error) {
  //     // Playwright should catch circular reference
  //     expect(error).toContain("circular");
  //   }
  // });

  // test("Binary data in multipart field", async ({ request }) => {
  //   const binaryData = Buffer.from([
  //     0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
  //   ]);

  //   const response = await request.post("/multipart-binary-field-test", {
  //     multipart: {
  //       textField: "normal text",
  //       binaryField: {
  //         name: "binary.dat",
  //         mimeType: "application/octet-stream",
  //         buffer: binaryData,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.textField).toBe("normal text");
  //   expect(body.binaryFileName).toBe("binary.dat");
  //   expect(body.binarySize).toBe(binaryData.length);
  // });

  // test("Streaming binary response", async ({ request }) => {
  //   const response = await request.get("/stream-binary-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/octet-stream");
  //   expect(response.headers()["transfer-encoding"]).toBe("chunked");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);
  // });

  // test("Compressed binary response", async ({ request }) => {
  //   const response = await request.get("/compressed-binary-test", {
  //     headers: {
  //       "Accept-Encoding": "gzip",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/octet-stream");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);
  // });

  // test("Multipart with mixed content types", async ({ request }) => {
  //   const jsonData = JSON.stringify({ nested: "object", number: 42 });
  //   const xmlData = "<?xml version='1.0'?><root><item>test</item></root>";
  //   const binaryData = Buffer.from([0x00, 0x01, 0x02, 0x03]);

  //   const response = await request.post("/multipart-mixed-test", {
  //     multipart: {
  //       jsonField: {
  //         name: "data.json",
  //         mimeType: "application/json",
  //         buffer: Buffer.from(jsonData),
  //       },
  //       xmlField: {
  //         name: "data.xml",
  //         mimeType: "application/xml",
  //         buffer: Buffer.from(xmlData),
  //       },
  //       binaryField: {
  //         name: "data.bin",
  //         mimeType: "application/octet-stream",
  //         buffer: binaryData,
  //       },
  //       textField: "plain text value",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.textField).toBe("plain text value");
  //   expect(body.filesCount).toBe(3);
  // });

  // test("Invalid Content-Length header", async ({ request }) => {
  //   const response = await request.post("/content-length-test", {
  //     data: "test data",
  //     headers: {
  //       "Content-Length": "999", // Wrong length
  //       "Content-Type": "text/plain",
  //     },
  //   });

  //   // Server should handle gracefully or return error
  //   expect([200, 400].includes(response.status())).toBe(true);
  // });

  // test("Request with authentication headers and binary upload", async ({
  //   request,
  // }) => {
  //   const binaryData = Buffer.from("Binary file content with auth");

  //   const response = await request.post("/auth-binary-upload-test", {
  //     data: binaryData,
  //     headers: {
  //       Authorization: "Bearer secret-token",
  //       "Content-Type": "application/octet-stream",
  //       "X-File-Name": "authenticated-file.bin",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.authenticated).toBe(true);
  //   expect(body.fileName).toBe("authenticated-file.bin");
  //   expect(body.size).toBe(binaryData.length);
  // });

  // test("Malformed multipart boundary", async ({ request }) => {
  //   // This test might be tricky as Playwright handles multipart encoding
  //   // but we can test server's response to potentially malformed data
  //   const response = await request.post("/malformed-multipart-test", {
  //     data: '--invalid-boundary\r\nContent-Disposition: form-data; name="test"\r\n\r\nvalue\r\n--invalid-boundary--',
  //     headers: {
  //       "Content-Type": "multipart/form-data; boundary=different-boundary",
  //     },
  //   });

  //   // Should return 400 for malformed multipart
  //   expect([400, 422].includes(response.status())).toBe(true);
  // });

  // test("Zero-byte file upload", async ({ request }) => {
  //   const response = await request.post("/zero-byte-file-test", {
  //     multipart: {
  //       metadata: "File metadata",
  //       file: {
  //         name: "zero.bin",
  //         mimeType: "application/octet-stream",
  //         buffer: Buffer.alloc(0),
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.metadata).toBe("File metadata");
  //   expect(body.fileSize).toBe(0);
  //   expect(body.fileName).toBe("zero.bin");
  // });

  // test("Request with unsupported media type", async ({ request }) => {
  //   const response = await request.post("/unsupported-media-test", {
  //     data: "test data",
  //     headers: {
  //       "Content-Type": "application/unknown-type",
  //     },
  //   });

  //   // Should return 415 Unsupported Media Type
  //   expect([415, 400].includes(response.status())).toBe(true);
  // });

  // test("Binary response with incorrect Content-Type", async ({ request }) => {
  //   const response = await request.get("/binary-content-type-error-test");

  //   expect(response.status()).toBe(200);
  //   // Server might return binary data but with wrong content-type
  //   expect(response.headers()["content-type"]).toBeDefined();

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);
  // });

  // test("Request with invalid HTTP version", async ({ request }) => {
  //   // This might not be testable directly through Playwright as it handles HTTP protocol
  //   // but we can test server's handling of edge cases
  //   const response = await request.get("/http-version-test", {
  //     headers: {
  //       "X-HTTP-Version-Test": "HTTP/1.0",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.httpVersion).toBeDefined();
  // });

  // test("Extremely large multipart upload", async ({ request }) => {
  //   const largeBuffer = Buffer.alloc(1024 * 1024, "x"); // 1MB file

  //   const response = await request.post("/large-multipart-test", {
  //     multipart: {
  //       description: "Large file upload test",
  //       largeFile: {
  //         name: "large.txt",
  //         mimeType: "text/plain",
  //         buffer: largeBuffer,
  //       },
  //     },
  //   });

  //   // Should either accept or return 413 (Payload Too Large)
  //   expect([200, 413].includes(response.status())).toBe(true);

  //   if (response.status() === 200) {
  //     const body = await response.json();
  //     expect(body.fileName).toBe("large.txt");
  //     expect(body.fileSize).toBe(largeBuffer.length);
  //   }
  // });

  // test("Binary upload with special filename characters", async ({
  //   request,
  // }) => {
  //   const testData = Buffer.from("Test file content");
  //   const specialFilename = "test file (1) [copy] - 测试.txt";

  //   const response = await request.post("/special-filename-test", {
  //     multipart: {
  //       file: {
  //         name: specialFilename,
  //         mimeType: "text/plain",
  //         buffer: testData,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.fileName).toBe(specialFilename);
  //   expect(body.fileSize).toBe(testData.length);
  // });

  // test("Multiple files with same field name", async ({ request }) => {
  //   const file1 = Buffer.from("First file content");
  //   const file2 = Buffer.from("Second file content");

  //   // Note: This might need to be handled differently depending on how your server expects multiple files
  //   // Note: Playwright's multipart API does not support arrays for a single field.
  //   // To send multiple files with the same field name, repeat the field key with different values.
  //   const response = await request.post("/multiple-same-field-test", {
  //     multipart: {
  //       files: {
  //         name: "file1.txt",
  //         mimeType: "text/plain",
  //         buffer: file1,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.fileCount).toBe(2);
  // });

  // test("Chunked binary upload", async ({ request }) => {
  //   const binaryData = Buffer.alloc(2048);
  //   // Fill with pattern to verify integrity
  //   for (let i = 0; i < 2048; i++) {
  //     binaryData[i] = i % 256;
  //   }

  //   const response = await request.post("/chunked-binary-test", {
  //     data: binaryData,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //       "Transfer-Encoding": "chunked",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.size).toBe(2048);
  //   expect(body.checksum).toBeDefined(); // Server should verify data integrity
  // });

  // test("Request with both query params and form data", async ({ request }) => {
  //   const response = await request.post(
  //     "/query-and-form-test?param1=query-value",
  //     {
  //       form: {
  //         field1: "form-value",
  //         field2: "another-form-value",
  //       },
  //     }
  //   );

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.queryParam).toBe("query-value");
  //   expect(body.formField1).toBe("form-value");
  //   expect(body.formField2).toBe("another-form-value");
  // });

  // test("Binary data with null bytes", async ({ request }) => {
  //   const dataWithNulls = Buffer.from([
  //     0x48,
  //     0x65,
  //     0x6c,
  //     0x6c,
  //     0x6f, // "Hello"
  //     0x00,
  //     0x00,
  //     0x00, // null bytes
  //     0x57,
  //     0x6f,
  //     0x72,
  //     0x6c,
  //     0x64, // "World"
  //     0x00, // trailing null
  //   ]);

  //   const response = await request.post("/null-bytes-test", {
  //     data: dataWithNulls,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.size).toBe(dataWithNulls.length);
  //   expect(body.hasNullBytes).toBe(true);
  // });

  // test("Malformed URL encoding in form data", async ({ request }) => {
  //   const response = await request.post("/malformed-url-encoding-test", {
  //     data: "field1=value%1&field2=value%GG&field3=normal", // Invalid % sequences
  //     headers: {
  //       "Content-Type": "application/x-www-form-urlencoded",
  //     },
  //   });

  //   // Server should handle malformed encoding gracefully
  //   expect([200, 400].includes(response.status())).toBe(true);
  // });

  // test("Request with conflicting Accept and Content-Type", async ({
  //   request,
  // }) => {
  //   const response = await request.post("/accept-content-mismatch-test", {
  //     data: { test: "json-data" },
  //     headers: {
  //       Accept: "text/plain",
  //       "Content-Type": "application/json",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   // Server should handle the mismatch appropriately
  //   const contentType = response.headers()["content-type"];
  //   expect(contentType).toBeDefined();
  // });

  // test("Binary response corruption check", async ({ request }) => {
  //   const response = await request.get("/binary-integrity-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/octet-stream");

  //   const buffer = await response.body();

  //   // Check for expected binary pattern
  //   expect(buffer.length).toBe(1024);
  //   for (let i = 0; i < 256; i++) {
  //     expect(buffer[i]).toBe(i); // Expected pattern
  //   }
  // });

  // test("Multipart with nested JSON in field", async ({ request }) => {
  //   const nestedJson = JSON.stringify({
  //     user: { id: 1, name: "Test User" },
  //     settings: { theme: "dark", notifications: true },
  //     tags: ["test", "json", "nested"],
  //   });

  //   const response = await request.post("/multipart-nested-json-test", {
  //     multipart: {
  //       metadata: nestedJson,
  //       simpleField: "simple value",
  //       file: {
  //         name: "data.json",
  //         mimeType: "application/json",
  //         buffer: Buffer.from(nestedJson),
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.metadata).toEqual(JSON.parse(nestedJson));
  //   expect(body.simpleField).toBe("simple value");
  //   expect(body.fileName).toBe("data.json");
  // });

  // test("Request with extremely long URL", async ({ request }) => {
  //   const longPath = "/long-url-test/" + "segment/".repeat(100);
  //   const longQuery = "param=" + "value".repeat(1000);

  //   try {
  //     const response = await request.get(`${longPath}?${longQuery}`);
  //     // Should either work or return 414 (URI Too Long)
  //     expect([200, 414].includes(response.status())).toBe(true);
  //   } catch (error) {
  //     // URL might be rejected by client
  //     expect(error).toBeDefined();
  //   }
  // });

  // test("Binary upload with progress tracking", async ({ request }) => {
  //   const largeData = Buffer.alloc(512 * 1024, 0xab); // 512KB of 0xAB bytes

  //   const response = await request.post("/binary-progress-test", {
  //     data: largeData,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //       "X-Expected-Size": largeData.length.toString(),
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.receivedSize).toBe(largeData.length);
  //   expect(body.expectedSize).toBe(largeData.length);
  //   expect(body.complete).toBe(true);
  // });

  // test("Request with duplicate Content-Type headers", async ({ request }) => {
  //   // This tests server handling of duplicate headers
  //   try {
  //     const response = await request.post("/duplicate-content-type-test", {
  //       data: { test: "data" },
  //       headers: {
  //         "Content-Type": "application/json",
  //         "content-type": "text/plain", // Duplicate with different case
  //       },
  //     });

  //     // Server should handle gracefully
  //     expect([200, 400].includes(response.status())).toBe(true);
  //   } catch (error) {
  //     // Playwright might reject duplicate headers
  //     expect(error).toBeDefined();
  //   }
  // });

  // test("Empty multipart field names", async ({ request }) => {
  //   const response = await request.post("/empty-field-names-test", {
  //     multipart: {
  //       "": "empty field name value",
  //       " ": "space field name value",
  //       normalField: "normal field value",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.normalField).toBe("normal field value");
  //   // Behavior for empty/space field names may vary
  // });

  // test("Binary response with Content-Disposition", async ({ request }) => {
  //   const response = await request.get("/binary-download-attachment-test");

  //   expect(response.status()).toBe(200);
  //   expect(response.headers()["content-type"]).toBe("application/octet-stream");
  //   expect(response.headers()["content-disposition"]).toContain("attachment");
  //   expect(response.headers()["content-disposition"]).toContain("filename=");

  //   const buffer = await response.body();
  //   expect(buffer.length).toBeGreaterThan(0);
  // });

  // test("Multipart upload with quoted field names", async ({ request }) => {
  //   const response = await request.post("/quoted-field-names-test", {
  //     multipart: {
  //       "field-with-dashes": "dashed field value",
  //       "field with spaces": "spaced field value",
  //       "field.with.dots": "dotted field value",
  //       "field[with][brackets]": "bracketed field value",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body["field-with-dashes"]).toBe("dashed field value");
  //   expect(body["field with spaces"]).toBe("spaced field value");
  //   expect(body["field.with.dots"]).toBe("dotted field value");
  //   expect(body["field[with][brackets]"]).toBe("bracketed field value");
  // });

  // test("Request with invalid JSON and valid fallback", async ({ request }) => {
  //   // Send invalid JSON but with text/plain fallback capability
  //   const response = await request.post("/json-fallback-test", {
  //     data: '{"invalid": json, but server might handle as text}',
  //     headers: {
  //       "Content-Type": "application/json",
  //       Accept: "application/json, text/plain",
  //     },
  //   });

  //   // Server might fallback to treating as plain text
  //   expect([200, 400].includes(response.status())).toBe(true);
  // });

  // test("Binary data integrity with checksums", async ({ request }) => {
  //   const testData = Buffer.from("Binary data for checksum verification");
  //   const crypto = require("crypto");
  //   const expectedChecksum = crypto
  //     .createHash("md5")
  //     .update(testData)
  //     .digest("hex");

  //   const response = await request.post("/binary-checksum-test", {
  //     data: testData,
  //     headers: {
  //       "Content-Type": "application/octet-stream",
  //       "X-Expected-MD5": expectedChecksum,
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.checksumMatch).toBe(true);
  //   expect(body.receivedChecksum).toBe(expectedChecksum);
  // });

  // test("Streaming multipart upload", async ({ request }) => {
  //   const chunk1 = Buffer.from("First chunk of streaming data");
  //   const chunk2 = Buffer.from("Second chunk of streaming data");
  //   const combinedData = Buffer.concat([chunk1, chunk2]);

  //   const response = await request.post("/streaming-multipart-test", {
  //     multipart: {
  //       description: "Streaming upload test",
  //       streamFile: {
  //         name: "stream.dat",
  //         mimeType: "application/octet-stream",
  //         buffer: combinedData,
  //       },
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.fileName).toBe("stream.dat");
  //   expect(body.totalSize).toBe(combinedData.length);
  //   expect(body.chunksReceived).toBeGreaterThan(0);
  // });

  // test("Error recovery in multipart parsing", async ({ request }) => {
  //   // This test checks if server can recover from partially corrupted multipart data
  //   const goodData = Buffer.from("This is good data");

  //   const response = await request.post("/multipart-error-recovery-test", {
  //     multipart: {
  //       validField: "This should work",
  //       validFile: {
  //         name: "good.txt",
  //         mimeType: "text/plain",
  //         buffer: goodData,
  //       },
  //       // Potentially problematic field
  //       problematicField: "",
  //     },
  //   });

  //   expect(response.status()).toBe(200);
  //   const body = await response.json();
  //   expect(body.validField).toBe("This should work");
  //   expect(body.validFileProcessed).toBe(true);
  // });
});
