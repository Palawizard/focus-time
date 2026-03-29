import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function TrackerScreen() {
  return (
    <Card className="ft-panel p-6">
      <CardHeader>
        <CardDescription>Tracker</CardDescription>
        <CardTitle>Tracked apps will appear here.</CardTitle>
      </CardHeader>

      <div className="mt-8 grid gap-3 md:grid-cols-2">
        <div className="ft-panel-muted px-4 py-3">
          <p className="ft-text-muted text-sm">Top app</p>
          <p className="mt-2 text-sm">None</p>
        </div>
        <div className="ft-panel-muted px-4 py-3">
          <p className="ft-text-muted text-sm">Exclusions</p>
          <p className="mt-2 text-sm">0 rules</p>
        </div>
      </div>
    </Card>
  );
}
