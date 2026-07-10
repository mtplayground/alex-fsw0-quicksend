export function App() {
  return (
    <main className="min-h-screen bg-slate-950 text-white">
      <section className="mx-auto flex min-h-screen w-full max-w-4xl flex-col justify-center px-6 py-16">
        <div className="max-w-2xl">
          <p className="mb-4 text-sm font-medium uppercase tracking-wide text-cyan-300">
            Frontend initialized
          </p>
          <h1 className="text-4xl font-semibold tracking-normal sm:text-5xl">
            React and Tailwind are ready.
          </h1>
          <p className="mt-6 max-w-xl text-base leading-7 text-slate-300">
            This placeholder page is served by the Axum backend from the production
            frontend build output.
          </p>
        </div>
      </section>
    </main>
  );
}
