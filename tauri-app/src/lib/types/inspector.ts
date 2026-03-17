export type LogStatus = "success" | "error" | "notification" | "pending";
export type RequestType = "server" | "client";

export interface InspectorEntry {
  id: string | number;
  timestamp: string;
  method: string;
  status: LogStatus;
  request: unknown;
  requestType: RequestType;
  response: unknown | null;
  stderr: string | null;
}

export interface McpConnection {
  id: string;
  name: string;
  transport: "stdio" | "sse" | "streamable-http";
}
