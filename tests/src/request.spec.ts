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
});
