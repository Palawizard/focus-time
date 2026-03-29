import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function StatsScreen() {
  return (
    <div className="grid gap-5 lg:grid-cols-3">
      <Card className="ft-panel p-6 lg:col-span-2">
        <CardHeader>
          <CardDescription>Stats</CardDescription>
          <CardTitle>La tendance apparaitra ici.</CardTitle>
        </CardHeader>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Cette semaine</CardDescription>
          <CardTitle>0 min</CardTitle>
        </CardHeader>
      </Card>
    </div>
  );
}
