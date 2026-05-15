# JavaScript Full-Stack Framework Guidance

Apply React, Next.js, Vue, Nuxt, Angular, Svelte, Express, and NestJS as delivery shells around explicit application logic.

- Keep pages, controllers, resolvers, and components thin: parse input, delegate to a use case, map the result.
- Separate presentational components from orchestration hooks, server actions, and domain services.
- Use query or cache libraries for server state; reserve global stores for truly global concerns such as session, theme, or tenant.
- Keep loading, error, empty, optimistic, and success states explicit in UI and API flows.
- Centralize auth, logging, and error mapping in middleware or framework filters without hiding business rules there.
- Do not leak framework request or response types into the domain model.
- Keep server and client boundaries explicit in hybrid frameworks such as Next.js and Nuxt.
