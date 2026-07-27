import type { Preview } from "@storybook/nextjs-vite";
import "../app/globals.css";

/**
 * Stories render against the real token stylesheet, and the theme is a
 * toolbar control so every component can be checked in both themes — the
 * design system's core promise.
 */
const preview: Preview = {
  parameters: {
    controls: { expanded: true },
    backgrounds: { disable: true },
  },
  globalTypes: {
    theme: {
      description: "Color theme",
      defaultValue: "dark",
      toolbar: {
        icon: "circlehollow",
        items: [
          { value: "light", title: "Light" },
          { value: "dark", title: "Dark" },
        ],
        dynamicTitle: true,
      },
    },
  },
  decorators: [
    (Story, context) => {
      const theme = context.globals.theme as string;
      if (typeof document !== "undefined") {
        document.documentElement.setAttribute("data-theme", theme);
      }
      return (
        <div className="bg-surface-page p-6 text-primary">
          <Story />
        </div>
      );
    },
  ],
};

export default preview;
