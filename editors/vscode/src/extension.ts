import * as fs from 'fs';
import * as path from 'path';
import * as lazy_stream from 'lazystream';
import { ChildProcess, spawn } from 'child_process';

import {
    workspace,
    ExtensionContext,
    TextDocument,
    WorkspaceConfiguration,
    WorkspaceFolder,
    window,
    commands,
    Uri,
} from 'vscode';

import {
    LanguageClient,
    LanguageClientOptions,
    Disposable,
    ServerOptions,
    RevealOutputChannelOn,
} from 'vscode-languageclient';

const DEFAULT_SERVER_EXE_NAME: string = 'oraide-language-server';
const workspaces: Map<Uri, ClientWorkspace> = new Map();

export async function activate(ctx: ExtensionContext) {
    workspace.onDidOpenTextDocument(doc => didOpenTextDocument(doc, ctx));
    workspace.textDocuments.forEach(doc => didOpenTextDocument(doc, ctx));
}

export async function deactivate() {
    return Promise.all([...workspaces.values()].map(ws => ws.stop()));
}

function didOpenTextDocument(document: TextDocument, ctx: ExtensionContext) {
    if (document.languageId !== 'miniyaml' && document.languageId !== 'yaml') {
        return;
    }

    const uri = document.uri;
    const dir = workspace.getWorkspaceFolder(uri);

    if (!dir) {
        console.error(`WorkspaceFolder \`${uri}\` could not be found`);
        return;
    }

    if (!workspaces.has(dir.uri)) {
        const ws = new ClientWorkspace(dir);
        workspaces.set(dir.uri, ws);
        ws.start(ctx);
    }
}

class ServerConfig {
    private readonly config: WorkspaceConfiguration;
    private readonly wsPath: string;

    private constructor(config: WorkspaceConfiguration, wsPath: string) {
        this.config = config;
        this.wsPath = wsPath;
    }

    public static loadFromWorkspace(wsPath: string): ServerConfig {
        const config = workspace.getConfiguration();
        return new ServerConfig(config, wsPath);
    }

    /**
     * If specified the server will be spawned by executing the file at the given path.
     */
    public get exePath(): string | undefined { return this.config.get<string>('oraide.server.exePath'); }
    public get shouldLogToFile(): boolean { return this.config.get<boolean>('oraide.server.shouldLogToFile', false); }
}

/**
 * We run one language server process and one corresponding language client per VSCode workspace directory.
 * This class contains all the per-client and per-workspace stuff.
 */
class ClientWorkspace {
    private readonly serverConfig: ServerConfig;
    private client: LanguageClient | null = null;
    private readonly dir: WorkspaceFolder;
    private disposables: Disposable[];

    private get dirFsPath() { return this.dir.uri.fsPath; }

    constructor(dir: WorkspaceFolder) {
        this.serverConfig = ServerConfig.loadFromWorkspace(dir.uri.fsPath);
        this.dir = dir;
        this.disposables = [];
    }

    public async start(ctx: ExtensionContext) {
        const serverOptions: ServerOptions = async () => this.spawnServerProcess();

        const clientOptions: LanguageClientOptions = {
            // Register the server for 'miniyaml' (.yaml) files
            documentSelector: [
                {
                    language: 'miniyaml',
                    scheme: 'file',
                    pattern: `${this.dirFsPath}/**/*`,
                },
                {
                    language: 'miniyaml',
                    scheme: 'untitled',
                    pattern: `${this.dirFsPath}/**/*`,
                },
            ],
            diagnosticCollectionName: 'OpenRA IDE',
            synchronize: { configurationSection: 'oraide' },
            workspaceFolder: this.dir,
            outputChannelName: "OpenRA IDE",
            revealOutputChannelOn: RevealOutputChannelOn.Info
        };

        this.client = new LanguageClient(
            'oraide',
            'OpenRA IDE',
            serverOptions,
            clientOptions,
        );

        this.registerCommands(ctx);
        this.disposables.push(this.client.start());
    }

    private async spawnServerProcess(): Promise<ChildProcess> {
        // Run the server executable from $PATH, unless there's an override.
        const serverExePath = this.serverConfig.exePath || DEFAULT_SERVER_EXE_NAME;

        const cwd = this.dirFsPath;
        const serverProcess = spawn(serverExePath, [], { cwd });

        serverProcess.on('error', (err: { code?: string; message: string }) => {
            if (err.code === 'ENOENT') {
                const msg = `Could not spawn \`${serverExePath}\`: ${err.message}`;
                console.error(msg);
                window.showWarningMessage(msg);
            }
        });

        if (this.serverConfig.shouldLogToFile) {
            const logPath = path.join(cwd, `oraide-${Date.now()}.log`);
            const logStream = new lazy_stream.Writable(() => fs.createWriteStream(logPath, { flags: 'w+' }));
            serverProcess.stderr.pipe(logStream);
        }

        return serverProcess;
    }

    private registerCommands(ctx: ExtensionContext) {
        if (!this.client) {
            return;
        }

        this.disposables.push(
            commands.registerCommand('oraide.server.restart', async () => {
                await this.stop();
                return this.start(ctx);
            })
        );
    }

    public async stop() {
        if (this.client) {
            await this.client.stop();
        }

        this.disposables.forEach(d => d.dispose());
    }
}