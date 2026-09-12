import type { Metadata } from "next";
import "./globals.css";
import { WalletProviders } from "@/components/WalletProviders";
import { AuthProvider } from "@/components/AuthProvider";
import { Navbar } from "@/components/Navbar";

export const metadata: Metadata = {
  title: "FPLX",
  description: "Fantasy sport, settled on-chain.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en">
      <body>
        <WalletProviders>
          <AuthProvider>
            <Navbar />
            <main className="wrap" style={{ paddingTop: 32, paddingBottom: 64 }}>
              {children}
            </main>
          </AuthProvider>
        </WalletProviders>
      </body>
    </html>
  );
}
