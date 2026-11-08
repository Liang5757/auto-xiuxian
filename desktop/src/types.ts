export interface LogEntry {
  id: number;
  time: string;
  message: string;
}
export interface Snapshot {
  dispatchRunning: boolean;
  dispatchStopping: boolean;
  dispatchStatus: string;
  map: string;
  success: number;
  failure: number;
  gameConnected: boolean;
  closing: boolean;
  logs: LogEntry[];
  clicker: {
    enabled: boolean;
    count: number;
    status: string;
    interval: number;
    right: boolean;
    key: number;
  };
}
export type Command =
  | { type: 'start_dispatch' | 'stop_dispatch' }
  | { type: 'set_clicker'; enabled: boolean }
  | { type: 'configure_clicker'; interval: number; right: boolean; key: number };
export type Page = 'dispatch' | 'clicker';
export interface DesktopState extends Snapshot {
  backendError: string | null;
}
export interface DesktopApi {
  getState(): Promise<Partial<DesktopState>>;
  command(command: Command): Promise<{ ok: boolean; error?: string }>;
  background(): void;
  onState(listener: (state: Partial<DesktopState>) => void): () => void;
}
declare global {
  interface Window {
    assistant?: DesktopApi;
  }
}
