import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

export function App() {
  return <SurfaceShell surface={surfaceById("UX-02")} />;
}
