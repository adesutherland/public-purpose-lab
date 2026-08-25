import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@public-purpose-lab/ui/styles.css";
import { App } from "./App.tsx";

const root = document.querySelector<HTMLDivElement>("#root");

if (!root) {
  throw new Error("Director root element was not found");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
