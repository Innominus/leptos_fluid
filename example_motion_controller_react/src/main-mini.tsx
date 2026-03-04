import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import AppMini from "./AppMini.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AppMini />
  </StrictMode>,
);
