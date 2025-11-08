// import { test, expect } from "@playwright/test";

// test.describe("WebSocket Server Tests", () => {
//   const WS_URL = "ws://127.0.0.1:3000/ws";

//   test.beforeAll(async () => {
//     // Add a small delay to ensure server is ready
//     await new Promise((resolve) => setTimeout(resolve, 1000));
//   });

//   test("should connect and receive welcome message", async ({ page }) => {
//     await page.evaluate((wsUrl) => {
//       return new Promise((resolve, reject) => {
//         const ws = new WebSocket(wsUrl);

//         ws.onopen = () => {
//           window.testWs = ws;
//           window.wsMessages = [];
//           window.wsConnected = true;
//           resolve(undefined);
//         };

//         ws.onmessage = (event) => {
//           window.wsMessages?.push(event.data);
//         };

//         ws.onerror = (error) => {
//           reject(error);
//         };

//         ws.onclose = () => {
//           window.wsConnected = false;
//         };
//       });
//     }, WS_URL);

//     // Wait for connection and initial message
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     const messages = await page.evaluate(() => window.wsMessages);
//     const isConnected = await page.evaluate(() => window.wsConnected);

//     expect(isConnected).toBe(true);
//     expect(messages![0]).toBe("Hello from ripress and wynd!");
//   });

//   test("should echo text messages", async ({ page }) => {
//     const testMessage = "Hello WebSocket Server!";

//     // Establish connection
//     await page.evaluate((wsUrl) => {
//       return new Promise((resolve, reject) => {
//         const ws = new WebSocket(wsUrl);

//         ws.onopen = () => {
//           window.testWs = ws;
//           window.wsMessages = [];
//           resolve(undefined);
//         };

//         ws.onmessage = (event) => {
//           window.wsMessages?.push(event.data);
//         };

//         ws.onerror = reject;
//       });
//     }, WS_URL);

//     // Wait for welcome message
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     // Clear messages and send test message
//     await page.evaluate(() => {
//       window.wsMessages = [];
//     });

//     await page.evaluate((message) => {
//       window.testWs?.send(message);
//     }, testMessage);

//     // Wait for echo response
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     console.log("here");

//     const messages = await page.evaluate(() => window.wsMessages);
//     expect(messages![0]).toBe(testMessage);
//   });

//   test("should handle multiple concurrent connections", async ({ browser }) => {
//     const contexts = await Promise.all([
//       browser.newContext(),
//       browser.newContext(),
//       browser.newContext(),
//     ]);

//     const pages = await Promise.all(contexts.map((ctx) => ctx.newPage()));

//     try {
//       // Connect all clients
//       await Promise.all(
//         pages.map((page) =>
//           page.evaluate((wsUrl) => {
//             return new Promise((resolve, reject) => {
//               const ws = new WebSocket(wsUrl);

//               ws.onopen = () => {
//                 window.testWs = ws;
//                 window.wsMessages = [];
//                 window.clientId = Math.random().toString(36).substr(2, 9);
//                 resolve(undefined);
//               };

//               ws.onmessage = (event) => {
//                 window.wsMessages?.push(event.data);
//               };

//               ws.onerror = reject;
//             });
//           }, WS_URL)
//         )
//       );

//       // Wait for all welcome messages
//       await Promise.all(
//         pages.map((page) =>
//           page.waitForFunction(
//             () => window.wsMessages && window.wsMessages.length > 0
//           )
//         )
//       );

//       // Send unique messages from each client
//       await Promise.all(
//         pages.map((page, index) =>
//           page.evaluate((index) => {
//             window.wsMessages = []; // Clear welcome message
//             const message = `Message from client ${index}`;
//             window.testMessage = message;
//             window.testWs?.send(message);
//           }, index)
//         )
//       );

//       // Verify each client receives its own echo
//       await Promise.all(
//         pages.map((page) =>
//           page.waitForFunction(
//             () => window.wsMessages && window.wsMessages.length > 0
//           )
//         )
//       );

//       const responses = await Promise.all(
//         pages.map((page) =>
//           page.evaluate(() => ({
//             sent: window.testMessage,
//             received: window.wsMessages![0],
//           }))
//         )
//       );

//       responses.forEach(({ sent, received }) => {
//         expect(received).toBe(sent);
//       });
//     } finally {
//       // Clean up
//       await Promise.all(contexts.map((ctx) => ctx.close()));
//     }
//   });

//   test("should handle connection close properly", async ({ page }) => {
//     let connectionClosed = false;

//     // Establish connection
//     await page.evaluate((wsUrl) => {
//       return new Promise((resolve, reject) => {
//         const ws = new WebSocket(wsUrl);

//         ws.onopen = () => {
//           window.testWs = ws;
//           window.wsMessages = [];
//           window.connectionClosed = false;
//           resolve(undefined);
//         };

//         ws.onmessage = (event) => {
//           window.wsMessages?.push(event.data);
//         };

//         ws.onclose = () => {
//           window.connectionClosed = true;
//         };

//         ws.onerror = reject;
//       });
//     }, WS_URL);

//     // Wait for welcome message
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     // Close connection
//     await page.evaluate(() => {
//       window.testWs?.close(1000, "Test close");
//     });

//     // Wait for close event
//     await page.waitForFunction(() => window.connectionClosed === true);

//     const isClosed = await page.evaluate(() => window.connectionClosed);
//     expect(isClosed).toBe(true);
//   });

//   test("should handle large messages", async ({ page }) => {
//     const largeMessage = "A".repeat(10000); // 10KB message

//     // Establish connection
//     await page.evaluate((wsUrl) => {
//       return new Promise((resolve, reject) => {
//         const ws = new WebSocket(wsUrl);

//         ws.onopen = () => {
//           window.testWs = ws;
//           window.wsMessages = [];
//           resolve(undefined);
//         };

//         ws.onmessage = (event) => {
//           window.wsMessages?.push(event.data);
//         };

//         ws.onerror = reject;
//       });
//     }, WS_URL);

//     // Wait for welcome message
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     // Clear messages and send large message
//     await page.evaluate(() => {
//       window.wsMessages = [];
//     });

//     await page.evaluate((message) => {
//       window.testWs?.send(message);
//     }, largeMessage);

//     // Wait for echo (with longer timeout for large message)
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0,
//       {
//         timeout: 10000,
//       }
//     );

//     const messages = await page.evaluate(() => window.wsMessages);
//     expect(messages![0]).toBe(largeMessage);
//     expect(messages![0]!.length).toBe(10000);
//   });

//   test("should maintain connection stability under rapid messages", async ({
//     page,
//   }) => {
//     // Establish connection
//     await page.evaluate((wsUrl) => {
//       return new Promise((resolve, reject) => {
//         const ws = new WebSocket(wsUrl);

//         ws.onopen = () => {
//           window.testWs = ws;
//           window.wsMessages = [];
//           window.messagesSent = 0;
//           resolve(undefined);
//         };

//         ws.onmessage = (event) => {
//           window.wsMessages?.push(event.data);
//         };

//         ws.onerror = reject;
//       });
//     }, WS_URL);

//     // Wait for welcome message
//     await page.waitForFunction(
//       () => window.wsMessages && window.wsMessages.length > 0
//     );

//     // Clear messages
//     await page.evaluate(() => {
//       window.wsMessages = [];
//     });

//     // Send rapid messages
//     const messageCount = 50;
//     await page.evaluate((count) => {
//       for (let i = 0; i < count; i++) {
//         window.testWs?.send(`Message ${i}`);
//         window.messagesSent!++;
//       }
//     }, messageCount);

//     // Wait for all echoes
//     await page.waitForFunction(
//       (expectedCount) =>
//         window.wsMessages && window.wsMessages.length >= expectedCount,
//       messageCount,
//       { timeout: 15000 }
//     );

//     const messages = await page.evaluate(() => window.wsMessages);
//     const sentCount = await page.evaluate(() => window.messagesSent);

//     expect(sentCount).toBe(messageCount);
//     expect(messages!.length).toBe(messageCount);

//     // Verify all messages were echoed correctly
//     for (let i = 0; i < messageCount; i++) {
//       expect(messages).toContain(`Message ${i}`);
//     }
//   });

//   test.afterEach(async ({ page }) => {
//     // Clean up WebSocket connections
//     await page
//       .evaluate(() => {
//         if (window.testWs && window.testWs.readyState === WebSocket.OPEN) {
//           window.testWs.close();
//         }
//       })
//       .catch(() => {
//         // Ignore errors during cleanup
//       });
//   });
// });
