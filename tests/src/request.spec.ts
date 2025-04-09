import { test, expect } from "@playwright/test";

test.describe("Request Tests", () => {
  test("Set and check cookies", async ({ request }) => {
    const cookieResponse = await request.get("/cookie-test", {
      headers: {
        Cookie: "sessionId=abc123",
      },
    });
    expect(cookieResponse.status()).toBe(200);
    const body = await cookieResponse.json();
    expect(body.sessionId === "abc123");
  });
  test("Set and check headers", async ({ request }) => {
    const headerResponse = await request.get("/header-test", {
      headers: {
        "Test-Header": "test-value",
      },
    });
    expect(headerResponse.status()).toBe(200);
    const body = await headerResponse.json();
    expect(body.header === "test-value");
  });
  test("Set and check params and query", async ({ request }) => {
    const paramAndQueryResponse = await request.get(
      "/param-and-query-test/test?query=test-query"
    );
    expect(paramAndQueryResponse.status()).toBe(200);
    const body = await paramAndQueryResponse.json();
    expect(body.param === "test");
    expect(body.query === "test-query");
  });

  test("Set and get origin_url and path", async ({ request }) => {
    const originUrlAndPathResponse = await request.get(
      "/origin-url-and-path/test?q=test"
    );
    expect(originUrlAndPathResponse.status()).toBe(200);
    const body = await originUrlAndPathResponse.json();
    expect(
      body.originUrl === "http://localhost:8080/origin-url-and-path/test?q=test"
    );
    expect(body.path === "/origin-url-and-path/test");
  });

  test("Test Ip", async ({ request }) => {
    const ipResponse = await request.get("/ip-test", {
      headers: {
        "X-Forwarded-For": "127.0.0.1",
      },
    });
    expect(ipResponse.status()).toBe(200);
    const body = await ipResponse.json();
    expect(body.ip === "127.0.0.1");
  });

  test("Set and get json body", async ({ request }) => {
    const jsonResponse = await request.post("/json-test", {
      data: { name: "test", age: 123 },
    });
    expect(jsonResponse.status()).toBe(200);
    const body = await jsonResponse.json();
    expect(body.name === "test");
    expect(body.age === 123);
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
    expect(body === "test");
  });

  test("Set and get form data", async ({ request }) => {
    const jsonResponse = await request.post("/form-test", {
      form: { name: "test" },
    });
    expect(jsonResponse.status()).toBe(200);
    const body = await jsonResponse.json();
    expect(body.name === "test");
  });

  test("Auth - Should be authenticated", async ({ request }) => {
    const authResponse = await request.get("/auth", {
      headers: {
        Cookie: "token=123abc",
      },
    });
    expect(authResponse.status()).toBe(200);
    const body = await authResponse.text();
    expect(body === "123abc");
  });

  test("Auth - isn't authenticated", async ({ request }) => {
    const authResponse = await request.get("/auth");
    expect(authResponse.status()).toBe(401);
    const body = await authResponse.text();
    expect(body === "unauthorized");
  });
});
