import "@jetbrains/ring-ui-built/components/style.css";
import "./styles.css";
import {StrictMode} from "react";
import {createRoot} from "react-dom/client";
import {BrowserRouter} from "react-router-dom";
import App from "./App";
import {loadRuntimeInfo} from "./api/client";

void loadRuntimeInfo().finally(() => {
  const root = document.getElementById("root");
  if (!root) {
    throw new Error("root element missing");
  }

  createRoot(root).render(
    <StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </StrictMode>
  );
});
