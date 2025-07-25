import {test, expect} from "@playwright/test";

test.describe("Response Tests", () => {
    test("Get Cookie", async ({request}) => {
        const cookieResponse = await request.get("/get-cookie-test");

        expect(cookieResponse.status()).toBe(200);

        const cookies = cookieResponse.headers()["set-cookie"];
        expect(cookies).toBeDefined();
        expect(cookies).toContain("test-cookie=value");
    });
});
