/** @type {import('next').NextConfig} */
const nextConfig = {
  // Mobile-first: set default viewport meta tag
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          {
            key: "viewport",
            value: "width=device-width, initial-scale=1, maximum-scale=5",
          },
        ],
      },
    ];
  },
};

module.exports = nextConfig;
