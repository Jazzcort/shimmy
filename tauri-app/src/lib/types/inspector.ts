export type LogStatus = "request" | "success" | "error" | "start";

export interface InspectorEntry {
	id: string;
	timestamp: string;
	method: string;
	status: LogStatus;
	request: unknown;
	response: unknown | null;
	stderr: string | null;
}

export interface McpConnection {
	id: string;
	name: string;
	transport: "stdio" | "sse" | "streamable-http";
}
