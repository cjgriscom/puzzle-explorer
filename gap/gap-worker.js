let sab = new SharedArrayBuffer(1024 * 1024);
let int32Array = new Int32Array(sab);
let uint8Array = new Uint8Array(sab);

// Send the SAB to the main thread immediately.
postMessage({ type: 'init_sab', sab });

self.Module = self.Module || {};
self.Module.arguments = ['-q', '--width', '1000000'];

self.Module.print = function (...args) {
  postMessage({ type: 'output', data: args.join(' ') + '\n' });
};

self.Module.printErr = function (...args) {
  postMessage({ type: 'error', data: args.join(' ') + '\n' });
};

self.Module.setStatus = function (text) {
  postMessage({ type: 'status', data: text });
};

self.Module.preRun = self.Module.preRun || [];
self.Module.preRun.push(() => {
  let inputBuffer = [];

  FS.init(
    () => {
      if (inputBuffer.length === 0) {
        // We need more input, block and wait for the main thread.
        postMessage({ type: 'read_request' });

        Atomics.store(int32Array, 0, 1); // State: waiting
        Atomics.wait(int32Array, 0, 1); // Wait until main thread changes state

        const len = Atomics.load(int32Array, 1);
        for (let i = 0; i < len; i++) {
          inputBuffer.push(Atomics.load(uint8Array, 8 + i));
        }

        Atomics.store(int32Array, 0, 0); // Reset state to idle
      }

      if (inputBuffer.length > 0) {
        return inputBuffer.shift();
      }
      return null;
    },
    null,
    null
  );
});

onmessage = (msg) => {
  // If the worker receives a message from the main thread directly (instead of via SAB),
  // we could handle it here. But usually the main thread writes to SAB and notifies.
};

async function loadAndStart() {
  const buffers = [];
  const totalParts = 6;

  for (let i = 0; i < totalParts; i++) {
    const url = `gap.data.part${i}`;
    try {
      // Add status message for loading parts
      self.Module.setStatus(`Downloading data part ${i + 1}/${totalParts}...`);

      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to load ${url}: ${response.status}`);
      }

      const buf = await response.arrayBuffer();
      buffers.push(new Uint8Array(buf));

      // Send progress update
      postMessage({ type: 'progress', data: (i + 1) / totalParts });

    } catch (e) {
      console.error(e);
      break;
    }
  }

  if (buffers.length > 0) {
    const totalLength = buffers.reduce((acc, b) => acc + b.length, 0);
    const mergedData = new Uint8Array(totalLength);
    let offset = 0;
    for (const buffer of buffers) {
      mergedData.set(buffer, offset);
      offset += buffer.length;
    }

    self.Module.getPreloadedPackage = function (remotePackageName) {
      if (remotePackageName === 'gap.data') {
        return mergedData.buffer;
      }
      return null;
    };

    self.Module.preRun.push(() => {
      try {
        FS.writeFile('/gap.data', mergedData);
      } catch (e) { }
    });
  } else {
    console.warn("Worker: No gap.data parts found.");
  }

  self.Module.setStatus("Initializing GAP...");

  // Load GAP
  importScripts("gap.js");

  // GAP is loaded, tell main thread it's ready!
  postMessage({ type: 'ready' });
  self.Module.setStatus(""); // clear status
}

loadAndStart();
