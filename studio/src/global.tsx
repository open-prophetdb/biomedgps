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

  // TODO: Does it matter if we don't clear the cache?
  // clearCache();
}
