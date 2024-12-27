import { createRoot } from "react-dom/client";
import { App } from "./app/app";
import "./app/theme/styles";

const container = document.getElementById("root");
if (!container) throw new Error("React root container not found");
const root = createRoot(container);
root.render(<App />);
