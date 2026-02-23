import { Link } from "react-router";
import { SectionWrapper } from "../components/ui/SectionWrapper";

export function NotFoundPage() {
  return (
    <SectionWrapper>
      <div className="min-h-[60vh] flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-6xl font-bold text-white mb-4">404</h1>
          <p className="text-lg text-surface-400 mb-8">Page not found</p>
          <Link
            to="/"
            className="inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 bg-primary-600 hover:bg-primary-500 text-white px-6 py-3"
          >
            Back to Home
          </Link>
        </div>
      </div>
    </SectionWrapper>
  );
}
