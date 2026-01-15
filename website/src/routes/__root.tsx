import { createRootRoute, Outlet, Link } from '@tanstack/react-router';
import { Github, Menu, X, Home, RefreshCw } from 'lucide-react';
import { useState } from 'react';

// 404 Not Found Component
function NotFoundComponent() {
  return (
    <div className="section min-h-[60vh] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold gradient-accent-text mb-4">404</h1>
        <p className="text-xl text-[var(--color-text-secondary)] mb-8">Page not found</p>
        <Link to="/" className="btn btn-primary">
          <Home className="w-4 h-4" />
          Back to Home
        </Link>
      </div>
    </div>
  );
}

// Error Component
function ErrorComponent({ error, reset }: { error: Error; reset: () => void }) {
  return (
    <div className="section min-h-[60vh] flex items-center justify-center">
      <div className="text-center max-w-md">
        <h1 className="text-4xl font-bold text-[var(--color-error)] mb-4">Something went wrong</h1>
        <p className="text-[var(--color-text-secondary)] mb-4">{error.message}</p>
        <button onClick={reset} className="btn btn-primary">
          <RefreshCw className="w-4 h-4" />
          Try Again
        </button>
      </div>
    </div>
  );
}

function RootComponent() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <>
      {/* Skip to content link for accessibility */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-[100] focus:px-4 focus:py-2 focus:bg-[var(--color-accent)] focus:text-[var(--color-void)] focus:rounded"
      >
        Skip to content
      </a>

      {/* Ambient Background */}
      <div className="ambient-backdrop" />
      <div className="grid-pattern" />

      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 nav-blur">
        <div className="container">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center gap-3 text-xl font-semibold">
              <img src="/logo.png" alt="FGP Logo" className="w-12 h-12 invert" />
              <span className="gradient-accent-text text-2xl">FGP</span>
            </Link>

            {/* Desktop Navigation */}
            <div className="hidden md:flex items-center gap-8">
              <Link
                to="/marketplace"
                className="nav-link"
              >
                Marketplace
              </Link>
              <Link
                to="/docs"
                className="nav-link"
              >
                Docs
              </Link>
              <a
                href="https://github.com/fast-gateway-protocol"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 nav-link"
              >
                <Github className="w-5 h-5" />
                GitHub
              </a>
              <Link to="/docs" className="btn btn-secondary btn-compact">
                Get started
              </Link>
            </div>

            {/* Mobile menu button */}
            <button
              className="md:hidden p-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              aria-label="Toggle menu"
            >
              {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </button>
          </div>
        </div>

        {/* Mobile Navigation */}
        {mobileMenuOpen && (
          <div className="md:hidden border-t border-[var(--color-border)] bg-[var(--color-void)]">
            <div className="container py-4 flex flex-col gap-4">
              <Link
                to="/marketplace"
                className="nav-link py-2"
                onClick={() => setMobileMenuOpen(false)}
              >
                Marketplace
              </Link>
              <Link
                to="/docs"
                className="nav-link py-2"
                onClick={() => setMobileMenuOpen(false)}
              >
                Docs
              </Link>
              <a
                href="https://github.com/fast-gateway-protocol"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 nav-link py-2"
              >
                <Github className="w-5 h-5" />
                GitHub
              </a>
              <Link to="/docs" className="btn btn-secondary btn-compact w-full justify-center">
                Get started
              </Link>
            </div>
          </div>
        )}
      </nav>

      {/* Main Content */}
      <main id="main-content" className="page-content pt-16">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="border-t border-[var(--color-border)] bg-[var(--color-surface)]">
        <div className="container py-12">
          <div className="flex flex-col md:flex-row justify-between items-center gap-6">
            <div className="flex items-center gap-3">
              <img src="/logo.png" alt="FGP Logo" className="w-8 h-8 invert opacity-60" />
              <span className="gradient-accent-text font-semibold text-lg">FGP</span>
              <span className="text-[var(--color-text-muted)]">Fast Gateway Protocol</span>
            </div>
            <div className="flex items-center gap-6 text-sm text-[var(--color-text-muted)]">
              <a
                href="https://github.com/fast-gateway-protocol"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-[var(--color-text-secondary)] transition-colors"
              >
                GitHub
              </a>
              <span>MIT License</span>
            </div>
          </div>
        </div>
      </footer>
    </>
  );
}

export const Route = createRootRoute({
  component: RootComponent,
  notFoundComponent: NotFoundComponent,
  errorComponent: ErrorComponent,
});
