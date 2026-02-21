# Delegant

Delegant is a powerful desktop application that gives an AI agent full access to your computer to automate tasks on your behalf. The agent can click, type, access files, and navigate the web autonomously based on your instructions.

Built as a lightweight desktop assistant, Delegant features a sleek, unobtrusive "Taskbar" interface alongside a robust Setup Wizard, making it easy to delegate complex workflows to an AI.

> **⚠️ Disclaimer:** Delegant provides an AI model with **full access to your device**. Use with caution. If the agent misbehaves or gets stuck, press **CTRL+ALT+DEL** or close the application immediately to suppress the agent.

## Tech Stack

- **Framework:** [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/)
- **Build Tool:** [Vite](https://vitejs.dev/)
- **Desktop Runtime:** [Tauri](https://tauri.app/) (v2)
- **State Management:** [Zustand](https://github.com/pmndrs/zustand)
- **Styling:** [Tailwind CSS 4](https://tailwindcss.com/)
- **Icons:** [Lucide React](https://lucide.dev/)

## Getting Started

### Prerequisites

Ensure you have the required dependencies for Tauri development on your operating system:

- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) (Node.js, Rust, and OS-specific build tools)

### Installation

1. Clone the repository:

   ```bash
   git clone <repository-url>
   cd delegant
   ```

2. Install dependencies:

   ```bash
   pnpm install
   ```

3. Run the development server (starts both Vite and the Tauri window):

   ```bash
   pnpm run tauri dev
   ```

### Building for Production

To build a production executable:

```bash
cargo tauri build
```

The resulting binaries will be located in `src-tauri/target/release/`.

## Architecture Overview

- `src/`: The React frontend, including components such as the Taskbar, Settings Wizard, Start Screen, and global Zustand stores.
- `src-tauri/`: The Rust backend responsible for integrating system-level events, executing actions, and acting as the bridge for the AI agent.

## License

MIT
