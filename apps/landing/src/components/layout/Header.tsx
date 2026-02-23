import { useState } from "react";
import { NavLink, Link } from "react-router";
import { content } from "../../data/content";
import { Button } from "../ui/Button";

export function Header() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const { nav } = content;

  const linkClass = ({ isActive }: { isActive: boolean }) =>
    `text-sm transition-colors nav-link ${isActive ? "text-white" : "text-surface-400 hover:text-white"}`;

  return (
    <header className="fixed top-0 left-0 right-0 z-50 border-b border-surface-800/50 bg-surface-950/80 backdrop-blur-md">
      <div className="mx-auto max-w-6xl flex items-center justify-between px-6 py-4">
        <Link to="/" className="text-xl font-bold text-white tracking-tight">
          OGBank
        </Link>

        <nav className="hidden md:flex items-center gap-8" aria-label="Main">
          {nav.links.map((link) => (
            <NavLink
              key={link.href}
              to={link.href}
              end
              className={linkClass}
            >
              {link.label}
            </NavLink>
          ))}
          <Button variant="primary" size="sm" href={nav.cta.href}>
            {nav.cta.label}
          </Button>
        </nav>

        <button
          className="md:hidden text-surface-400 hover:text-white"
          onClick={() => setMobileOpen(!mobileOpen)}
          aria-label="Toggle menu"
          aria-expanded={mobileOpen}
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            {mobileOpen ? (
              <path d="M6 6l12 12M6 18L18 6" />
            ) : (
              <path d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {mobileOpen && (
        <nav className="md:hidden border-t border-surface-800/50 px-6 py-4 bg-surface-950/95 backdrop-blur-md">
          {nav.links.map((link) => (
            <NavLink
              key={link.href}
              to={link.href}
              end
              className={({ isActive }) =>
                `block py-3 transition-colors ${isActive ? "text-white" : "text-surface-300 hover:text-white"}`
              }
              onClick={() => setMobileOpen(false)}
            >
              {link.label}
            </NavLink>
          ))}
          <div className="pt-3">
            <Button variant="primary" size="sm" href={nav.cta.href}>
              {nav.cta.label}
            </Button>
          </div>
        </nav>
      )}
    </header>
  );
}
