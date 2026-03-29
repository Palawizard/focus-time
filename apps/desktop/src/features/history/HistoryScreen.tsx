import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function HistoryScreen() {
  return (
    <Card className="ft-panel p-6">
      <CardHeader>
        <CardDescription>History</CardDescription>
        <CardTitle>Retrouve tes sessions passees.</CardTitle>
      </CardHeader>

      <div className="mt-8 grid gap-3">
        <div className="ft-panel-muted flex items-center justify-between px-4 py-3">
          <span className="ft-text-muted text-sm">Aucune session recente.</span>
          <span className="text-sm">0 min</span>
        </div>
      </div>
    </Card>
  );
}
