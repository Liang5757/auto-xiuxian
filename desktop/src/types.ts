export interface LogEntry {
  id: number;
  time: string;
  message: string;
}
export interface Snapshot {
  dispatchInterval: number;
  farm: {
    interval: number;
    running: boolean;
    stopping: boolean;
    status: string;
    map: string;
    success: number;
    failure: number;
    logs: LogEntry[];
  };
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
  | {
      type:
        | 'stop_both'
        | 'start_both'
        | 'stop_farm'
        | 'start_farm'
        | 'start_dispatch'
        | 'stop_dispatch';
    }
  | { type: 'configure_polling'; farming: boolean; interval: number }
  | { type: 'set_clicker'; enabled: boolean }
  | { type: 'configure_clicker'; interval: number; right: boolean; key: number };
export type Page = 'farm' | 'dispatch' | 'clicker';
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
