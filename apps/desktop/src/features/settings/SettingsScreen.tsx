import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function SettingsScreen() {
  return (
    <Card className="ft-panel p-6">
      <CardHeader>
        <CardDescription>Settings</CardDescription>
        <CardTitle>Shape the flow that works for you.</CardTitle>
      </CardHeader>

      <div className="mt-8 grid gap-3 md:grid-cols-3">
        <div className="ft-panel-muted px-4 py-3">
          <p className="ft-text-muted text-sm">Focus</p>
          <p className="mt-2 text-sm">25 min</p>
        </div>
        <div className="ft-panel-muted px-4 py-3">
          <p className="ft-text-muted text-sm">Break</p>
          <p className="mt-2 text-sm">5 min</p>
        </div>
        <div className="ft-panel-muted px-4 py-3">
          <p className="ft-text-muted text-sm">Theme</p>
          <p className="mt-2 text-sm">System</p>
        </div>
      </div>
    </Card>
  );
}
