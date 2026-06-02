import { defineConfig } from "vitepress";
import { withMermaid } from "vitepress-plugin-mermaid";

export default withMermaid(defineConfig({
  base: "/boundline/",
  title: "Boundline",
  description:
    "Bounded cognitive runtime for AI-assisted software delivery: planning, execution, review, governance, and explainable orchestration.",

  cleanUrls: true,
  lastUpdated: true,
  ignoreDeadLinks: true,

  head: [
    ["meta", { name: "theme-color", content: "#070412" }],
    ["link", { rel: "icon", href: "/images/boundline-icon.svg" }],
    ["link", { rel: "stylesheet", href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css" }]
  ],

  themeConfig: {
    logo: "/images/boundline-icon.svg",

    nav: [
      { text: "Guide", link: "/guide/introduction" },
      { text: "Architecture", link: "/architecture/runtime-model" },
      { text: "Reference", link: "/reference/cli" },
      { text: "Roadmap", link: "/roadmap/" },
      { text: "GitHub", link: "https://github.com/apply-the/boundline" }
    ],

    sidebar: {
      "/guide/": [
        {
          text: "Guide",
          items: [
            { text: "Introduction", link: "/guide/introduction" },
            { text: "Constitution", link: "/guide/constitution" },
            { text: "Getting Started", link: "/guide/getting-started" },
            { text: "Installation", link: "/guide/installation" },
            { text: "First Workspace", link: "/guide/first-workspace" },
            { text: "Core Concepts", link: "/guide/core-concepts" },
            { text: "Common Workflows", link: "/guide/common-workflows" }
          ]
        }
      ],
      "/runtime/": [
        {
          text: "Runtime",
          items: [
            { text: "Goal", link: "/runtime/goal" },
            { text: "Plan", link: "/runtime/plan" },
            { text: "Run", link: "/runtime/run" },
            { text: "Status", link: "/runtime/status" },
            { text: "Inspect", link: "/runtime/inspect" },
            { text: "Trace", link: "/runtime/trace" },
            { text: "Phase Requests", link: "/runtime/phase-requests" },
            { text: "Stop Semantics", link: "/runtime/stop-semantics" }
          ]
        }
      ],
      "/governance/": [
        {
          text: "Governance",
          items: [
            { text: "Canon-Aware Defaults", link: "/governance/canon-aware-defaults" },
            { text: "Guidance", link: "/governance/guidance" },
            { text: "Guardians", link: "/governance/guardians" },
            { text: "Evidence", link: "/governance/evidence" },
            { text: "Review Gates", link: "/governance/review-gates" }
          ]
        }
      ],
      "/adapters/": [
        {
          text: "Adapters",
          items: [
            { text: "Overview", link: "/adapters/overview" },
            { text: "Registration", link: "/adapters/registration" },
            { text: "Speckit", link: "/adapters/speckit" },
            { text: "Custom Adapters", link: "/adapters/custom-adapters" },
            { text: "Protocol", link: "/adapters/protocol" },
            { text: "Stage Ownership", link: "/adapters/stage-ownership" },
            { text: "Hooks", link: "/adapters/hooks" },
            { text: "Troubleshooting", link: "/adapters/troubleshooting" }
          ]
        }
      ],
      "/architecture/": [
        {
          text: "Architecture",
          items: [
            { text: "Runtime Model", link: "/architecture/runtime-model" },
            { text: "Session Model", link: "/architecture/session-model" },
            { text: "Context Intelligence", link: "/architecture/context-intelligence" },
            { text: "Recursive Stage Refinement", link: "/architecture/recursive-stage-refinement" },
            { text: "Persistent Stdio", link: "/architecture/persistent-stdio" },
            { text: "Security Model", link: "/architecture/security-model" }
          ]
        }
      ],
      "/reference/": [
        {
          text: "Reference",
          items: [
            { text: "CLI Reference", link: "/reference/cli" },
            { text: "Configuration", link: "/reference/configuration" },
            { text: "File Layout", link: "/reference/file-layout" },
            { text: "Glossary", link: "/reference/glossary" },
            { text: "FAQ", link: "/reference/faq" }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/apply-the/boundline" }
    ],

    search: {
      provider: "local"
    },

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © Apply The"
    }
  }
}));
