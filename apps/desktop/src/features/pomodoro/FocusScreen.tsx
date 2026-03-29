import { Button } from "../../components/ui/Button";
import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";

export function FocusScreen() {
  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1.35fr)_minmax(21rem,1fr)]">
      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Focus</CardDescription>
          <CardTitle>Prends un temps pour avancer.</CardTitle>
        </CardHeader>

        <div className="mt-8 grid gap-4 md:grid-cols-3">
          <Card>
            <CardDescription>Session</CardDescription>
            <CardTitle className="mt-3">25 min</CardTitle>
          </Card>
          <Card>
            <CardDescription>Pause</CardDescription>
            <CardTitle className="mt-3">5 min</CardTitle>
          </Card>
          <Card>
            <CardDescription>Cycles</CardDescription>
            <CardTitle className="mt-3">4</CardTitle>
          </Card>
        </div>

        <div className="mt-8 flex flex-wrap gap-3">
          <Button>Demarrer</Button>
          <Button variant="secondary">Preset</Button>
        </div>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Session en cours</CardDescription>
          <CardTitle>Aucune session active.</CardTitle>
        </CardHeader>
      </Card>
    </div>
  );
}
