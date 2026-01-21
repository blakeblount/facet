import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:3001',
				changeOrigin: true
			}
		}
	},
	plugins: [
		sveltekit(),
		SvelteKitPWA({
			srcDir: 'src',
			strategies: 'generateSW',
			registerType: 'prompt',
			manifest: false, // We use our own static manifest.json
			injectRegister: false, // We'll handle registration in our component
			workbox: {
				globPatterns: ['**/*.{js,css,html,ico,png,svg,woff,woff2}'],
				runtimeCaching: [
					{
						// SvelteKit data endpoints - always fetch fresh for real-time updates
						urlPattern: /\/__data\.json/,
						handler: 'NetworkFirst',
						options: {
							cacheName: 'sveltekit-data',
							networkTimeoutSeconds: 5,
							expiration: {
								maxEntries: 50,
								maxAgeSeconds: 60 // 1 minute - short cache for fallback only
							},
							cacheableResponse: {
								statuses: [0, 200]
							}
						}
					},
					{
						urlPattern: /^https:\/\/.*\.(?:png|jpg|jpeg|svg|gif|webp)$/,
						handler: 'CacheFirst',
						options: {
							cacheName: 'images-cache',
							expiration: {
								maxEntries: 100,
								maxAgeSeconds: 30 * 24 * 60 * 60 // 30 days
							}
						}
					},
					{
						urlPattern: /^https?:\/\/.*\/api\/v1\/.*/,
						handler: 'NetworkFirst',
						options: {
							cacheName: 'api-cache',
							networkTimeoutSeconds: 10,
							expiration: {
								maxEntries: 50,
								maxAgeSeconds: 24 * 60 * 60 // 24 hours
							},
							cacheableResponse: {
								statuses: [0, 200]
							}
						}
					}
				]
			},
			devOptions: {
				enabled: true,
				type: 'module'
			}
		})
	]
});
