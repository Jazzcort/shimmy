import type {
  JSONRPCNotification,
  JSONRPCRequest,
  JSONRPCResponse,
} from "@modelcontextprotocol/sdk/types.js";

export interface McpInitializeFinish {
  serverId: string;
  requestId: number | string;
}

export interface StampedMcpRequest {
  request: JSONRPCRequest;
  timestamp: string;
}

export interface StampedMcpResponse {
  response: JSONRPCResponse;
  timestamp: string;
}

export interface StampedMcpNotification {
  notification: JSONRPCNotification;
  timestamp: string;
}

export interface IncomingRequest {
  serverId: string;
  requestId: number | string;
}

export interface IncomingResponse {
  serverId: string;
  responseId: number | string;
}

export interface IncomingNotification {
  serverId: string;
  notificationId: number | string;
}
