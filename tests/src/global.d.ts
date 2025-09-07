// Global type definitions for WebSocket test environment

declare global {
  interface Window {
    // WebSocket connection instance
    testWs?: WebSocket;

    // Message storage arrays
    wsMessages?: string[];
    binaryMessages?: ArrayBuffer[];

    // Connection state flags
    wsConnected?: boolean;
    connectionClosed?: boolean;
    upgradeSuccessful?: boolean;
    handshakeSuccessful?: boolean;
    serverClosed?: boolean;

    errors: Events[];

    // Connection metadata
    clientId?: string;
    connectionTime?: number;
    messageCount?: number;
    lastMessageTime?: number;

    // Close event data
    closeCode?: number;
    closeReason?: string;
    serverCloseCode?: number;
    closeReceived?: boolean;

    // Protocol and extension data
    protocol?: string;
    extensions?: string;

    // Performance measurement
    startTime?: number;
    latencies?: number[];

    // Test-specific data
    testMessage?: string;
    messagesSent?: number;
    burstNumber?: number;
    pingReceived?: boolean;

    // Error handling
    wsError?: any;
  }

  // Extend globalThis to include our test properties
  interface GlobalThis {
    wsMessages?: string[];
    wsConnected?: boolean;
    messageCount?: number;
    lastMessageTime?: number;
  }
}

export {};
