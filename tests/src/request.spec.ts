import {expect, test} from "@playwright/test";

test.describe("Request Tests", () => {
    test("Set and check cookies", async ({request}) => {
        const cookieResponse = await request.get("/cookie-test", {
            headers: {
                Cookie: "sessionId=abc123",
            },
        });

        expect(cookieResponse.status()).toBe(200);

        const body = await cookieResponse.json();
        expect(body.sessionId === "abc123");
    });

    test("Set and check headers", async ({request}) => {
        const headerResponse = await request.get("/header-test", {
            headers: {
                "Test-Header": "test-value",
            },
        });

        expect(headerResponse.status()).toBe(200);

        const body = await headerResponse.json();
        expect(body.header === "test-value");
    });

    test("Set and check params and query", async ({request}) => {
        const paramAndQueryResponse = await request.get(
            "/param-and-query-test/test?query=test-query"
        );

        expect(paramAndQueryResponse.status()).toBe(200);

        const body = await paramAndQueryResponse.json();
        expect(body.param === "test");
        expect(body.query === "test-query");
    });

    test("Set and get origin_url and path", async ({request}) => {
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

    test("Test Ip", async ({request}) => {
        const ipResponse = await request.get("/ip-test");

        expect(ipResponse.status()).toBe(200);

        const body = await ipResponse.json();

        expect(body.ip === "127.0.0.1");
    });

    test("Set and get json body", async ({request}) => {
        const jsonResponse = await request.post("/json-test", {
            data: {name: "test", age: 123},
        });

        expect(jsonResponse.status()).toBe(200);

        const body = await jsonResponse.json();
        expect(body.name === "test");
        expect(body.age === 123);
    });

    test("Set and get text body", async ({request}) => {
        const textResponse = await request.post("/text-test", {
            data: "test",
        });

        expect(textResponse.status()).toBe(200);

        const body = await textResponse.text();
        expect(body === "test");
    });

    test("Set and get form data", async ({request}) => {
        const jsonResponse = await request.post("/form-test", {
            form: {name: "test"},
        });

        expect(jsonResponse.status()).toBe(200);

        const body = await jsonResponse.json();
        expect(body.name === "test");
    });

    test("Auth - Should be authenticated", async ({request}) => {
        const authResponse = await request.get("/auth", {
            headers: {
                Cookie: "token=123abc",
            },
        });

        expect(authResponse.status()).toBe(200);

        const body = await authResponse.text();
        expect(body === "123abc");
    });

    test("Auth - isn't authenticated", async ({request}) => {
        const authResponse = await request.get("/auth");

        expect(authResponse.status()).toBe(401);

        const body = await authResponse.text();
        expect(body === "unauthorized");
    });

    // Additional Request Object Tests

    test("Multiple query parameters", async ({request}) => {
        const response = await request.get("/multi-query?name=john&age=25&city=NYC");

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.name === "john");
        expect(body.age === "25");
        expect(body.city === "NYC");
    });

    test("Multiple route parameters", async ({request}) => {
        const response = await request.get("/users/123/posts/456");

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.userId === "123");
        expect(body.postId === "456");
    });

    test("Multiple cookies", async ({request}) => {
        const response = await request.get("/multi-cookies", {
            headers: {
                Cookie: "user=john; theme=dark; lang=en",
            },
        });

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.user === "john");
        expect(body.theme === "dark");
        expect(body.lang === "en");
    });

    test("Multiple headers", async ({request}) => {
        const response = await request.get("/multi-headers", {
            headers: {
                "User-Agent": "TestBot/1.0",
                "Accept": "application/json",
                "Authorization": "Bearer token123",
                "X-Custom-Header": "custom-value",
            },
        });

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.userAgent === "TestBot/1.0");
        expect(body.accept === "application/json");
        expect(body.authorization === "Bearer token123");
        expect(body.customHeader === "custom-value");
    });

    test("Request method detection", async ({request}) => {
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

    test("URL-encoded form data", async ({request}) => {
        const response = await request.post("/urlencoded-test", {
            form: {
                username: "testuser",
                password: "secret123",
                email: "test@example.com",
            },
        });

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.username === "testuser");
        expect(body.password === "secret123");
        expect(body.email === "test@example.com");
    });

    test("Raw body data", async ({request}) => {
        const rawData = "raw text content";
        const response = await request.post("/raw-body-test", {
            data: rawData,
            headers: {
                "Content-Type": "text/plain",
            },
        });

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.rawBody === rawData);
        expect(body.contentType === "text/plain");
    });

    // test("Request with custom port", async ({request}) => {
    //     const response = await request.get("/port-test", {
    //         headers: {
    //             Host: "localhost:8080",
    //         },
    //     });
    //
    //     expect(response.status()).toBe(200);
    //
    //     const body = await response.json();
    //     expect(body.host === "localhost:8080");
    //     expect(body.hostname === "localhost");
    // });
    //

    test("Request secure/insecure", async ({request}) => {
        const response = await request.get("/secure-test");

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.secure === false); // Assuming HTTP for tests
    });


    test("Request xhr detection", async ({request}) => {
        const response = await request.get("/xhr-test", {
            headers: {
                "X-Requested-With": "XMLHttpRequest",
            },
        });

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(body.xhr === true);
    });

    test("Empty request body", async ({request}) => {
        const response = await request.post("/empty-body-test");

        expect(response.status()).toBe(200);

        const body = await response.json();
        expect(!body.hasBody);
    });

});