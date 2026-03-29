import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function StatsScreen() {
  return (
    <div className="grid gap-5 lg:grid-cols-3">
      <Card className="ft-panel p-6 lg:col-span-2">
        <CardHeader>
          <CardDescription>Stats</CardDescription>
          <CardTitle>Your trend will show up here.</CardTitle>
        </CardHeader>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>This week</CardDescription>
          <CardTitle>0 min</CardTitle>
        </CardHeader>
      </Card>
    </div>
  );
}
