import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function GamificationScreen() {
  return (
    <div className="grid gap-5 lg:grid-cols-3">
      <Card className="ft-panel p-6 lg:col-span-2">
        <CardHeader>
          <CardDescription>Gamification</CardDescription>
          <CardTitle>Ta progression sera visible ici.</CardTitle>
        </CardHeader>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Serie</CardDescription>
          <CardTitle>0 jour</CardTitle>
        </CardHeader>
      </Card>
    </div>
  );
}
