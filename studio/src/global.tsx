import FingerprintJS from '@fingerprintjs/fingerprintjs';

const getIdentity = async () => {
  let visitorId = localStorage.getItem('rapex-visitor-id')

  if (!visitorId) {
    const fpPromise = FingerprintJS.load();
    // Get the visitor identifier when you need it.
    const fp = await fpPromise
    const result = await fp.get()

    visitorId = result.visitorId
  }

  return visitorId
}

const visitorId = await getIdentity();
localStorage.setItem('rapex-visitor-id', visitorId);

const isHttps = document.location.protocol === 'https:';

const clearCache = () => {
  // Remove all caches but not the webllm caches
  const cachedKeys = ['webllm/config', 'webllm/wasm', 'webllm/model']
  if (window.caches) {
    caches
      .keys()
      .then((keys) => {
        keys.forEach((key) => {
          if (cachedKeys.includes(key)) return;
          console.log('clear caches', keys);
          caches.delete(key);
        });
      })
      .catch((e) => console.log(e));
  }
};

if ('serviceWorker' in navigator && isHttps) {
  // unregister service worker
  const { serviceWorker } = navigator;
  if (serviceWorker.getRegistrations) {
    serviceWorker.getRegistrations().then((sws) => {
      sws.forEach((sw) => {
        sw.unregister();
      });
    });
  }
  serviceWorker.getRegistration().then((sw) => {
    if (sw) sw.unregister();
  });

  clearCache();
}
