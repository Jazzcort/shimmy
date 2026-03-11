import type { InspectorEntry, McpConnection } from "$lib/types/inspector";

export const connections: McpConnection[] = [
	{ id: "conn-1", name: "filesystem-server", transport: "stdio" },
	{ id: "conn-2", name: "github-mcp", transport: "sse" },
	{ id: "conn-3", name: "database-server", transport: "streamable-http" },
];

export const entries: InspectorEntry[] = [
	{
		id: "entry-1",
		timestamp: "14:23:01.123",
		method: "initialize",
		status: "start",
		request: {
			jsonrpc: "2.0",
			id: 1,
			method: "initialize",
			params: {
				protocolVersion: "2024-11-05",
				capabilities: {
					roots: { listChanged: true },
					sampling: {},
				},
				clientInfo: {
					name: "shimmy",
					version: "0.1.0",
				},
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 1,
			result: {
				protocolVersion: "2024-11-05",
				capabilities: {
					tools: { listChanged: true },
					resources: { subscribe: true, listChanged: true },
				},
				serverInfo: {
					name: "filesystem-server",
					version: "1.2.0",
				},
			},
		},
		stderr: null,
	},
	{
		id: "entry-2",
		timestamp: "14:23:01.456",
		method: "notifications/initialized",
		status: "request",
		request: {
			jsonrpc: "2.0",
			method: "notifications/initialized",
		},
		response: null,
		stderr: null,
	},
	{
		id: "entry-3",
		timestamp: "14:23:02.789",
		method: "tools/list",
		status: "success",
		request: {
			jsonrpc: "2.0",
			id: 2,
			method: "tools/list",
		},
		response: {
			jsonrpc: "2.0",
			id: 2,
			result: {
				tools: [
					{
						name: "read_file",
						description: "Read the complete contents of a file",
						inputSchema: {
							type: "object",
							properties: {
								path: { type: "string", description: "Path to the file" },
							},
							required: ["path"],
						},
					},
					{
						name: "write_file",
						description: "Write contents to a file",
						inputSchema: {
							type: "object",
							properties: {
								path: { type: "string" },
								content: { type: "string" },
							},
							required: ["path", "content"],
						},
					},
					{
						name: "list_directory",
						description: "List contents of a directory",
						inputSchema: {
							type: "object",
							properties: {
								path: { type: "string" },
							},
							required: ["path"],
						},
					},
				],
			},
		},
		stderr: null,
	},
	{
		id: "entry-4",
		timestamp: "14:23:03.012",
		method: "resources/list",
		status: "success",
		request: {
			jsonrpc: "2.0",
			id: 3,
			method: "resources/list",
		},
		response: {
			jsonrpc: "2.0",
			id: 3,
			result: {
				resources: [
					{
						uri: "file:///project/README.md",
						name: "README.md",
						mimeType: "text/markdown",
					},
				],
			},
		},
		stderr: null,
	},
	{
		id: "entry-5",
		timestamp: "14:23:05.234",
		method: "tools/call",
		status: "success",
		request: {
			jsonrpc: "2.0",
			id: 4,
			method: "tools/call",
			params: {
				name: "read_file",
				arguments: {
					path: "/project/src/index.ts",
				},
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 4,
			result: {
				content: [
					{
						type: "text",
						text: 'import express from "express";\n\nconst app = express();\napp.listen(3000);',
					},
				],
			},
		},
		stderr: "Warning: file is 4 lines long\n",
	},
	{
		id: "entry-6",
		timestamp: "14:23:06.789",
		method: "tools/call",
		status: "error",
		request: {
			jsonrpc: "2.0",
			id: 5,
			method: "tools/call",
			params: {
				name: "read_file",
				arguments: {
					path: "/project/nonexistent.ts",
				},
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 5,
			error: {
				code: -32602,
				message: "File not found: /project/nonexistent.ts",
			},
		},
		stderr: "Error: ENOENT: no such file or directory\n",
	},
	{
		id: "entry-7",
		timestamp: "14:23:08.456",
		method: "tools/call",
		status: "success",
		request: {
			jsonrpc: "2.0",
			id: 6,
			method: "tools/call",
			params: {
				name: "list_directory",
				arguments: {
					path: "/project/src",
				},
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 6,
			result: {
				content: [
					{
						type: "text",
						text: "index.ts\nutils.ts\nconfig.ts",
					},
				],
			},
		},
		stderr: null,
	},
	{
		id: "entry-8",
		timestamp: "14:23:10.123",
		method: "resources/read",
		status: "request",
		request: {
			jsonrpc: "2.0",
			id: 7,
			method: "resources/read",
			params: {
				uri: "file:///project/README.md",
			},
		},
		response: null,
		stderr: null,
	},
	{
		id: "entry-9",
		timestamp: "14:23:10.456",
		method: "resources/read",
		status: "success",
		request: {
			jsonrpc: "2.0",
			id: 7,
			method: "resources/read",
			params: {
				uri: "file:///project/README.md",
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 7,
			result: {
				contents: [
					{
						uri: "file:///project/README.md",
						mimeType: "text/markdown",
						text: "# My Project\n\nA sample project for testing MCP.",
					},
				],
			},
		},
		stderr: null,
	},
	{
		id: "entry-10",
		timestamp: "14:23:12.789",
		method: "tools/call",
		status: "error",
		request: {
			jsonrpc: "2.0",
			id: 8,
			method: "tools/call",
			params: {
				name: "write_file",
				arguments: {
					path: "/readonly/file.txt",
					content: "test",
				},
			},
		},
		response: {
			jsonrpc: "2.0",
			id: 8,
			error: {
				code: -32603,
				message: "Permission denied: cannot write to /readonly/file.txt",
			},
		},
		stderr: "Error: EACCES: permission denied, open '/readonly/file.txt'\n",
	},
];
