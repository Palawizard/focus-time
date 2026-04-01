export interface RuntimeHealth {
  productName: string;
  appVersion: string;
  desktopShell: string;
  platform: string;
  persistenceMode: string;
  workspaceCrates: string[];
  appDataDir: string;
  appLogDir: string;
  backupDir: string;
  launchOnStartupEnabled: boolean;
  trayEnabled: boolean;
  closeToTray: boolean;
}
