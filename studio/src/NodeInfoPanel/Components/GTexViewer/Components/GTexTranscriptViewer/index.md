---
title: GTexTranscriptViewer
group:
  path: /components/visualization-components
  title: Visualization
---

## GTexTranscriptViewer

### IsoformTransposed Type

```tsx
import React from 'react';
import { GTexTranscriptViewer } from 'gtex-d3';

export default () => (
  <GTexTranscriptViewer
    rootId="gtex-transcript-viewer-isoform-transposed"
    title="GTex Transcript Viewer"
    geneId="ENSG00000141510"
    type="isoformTransposed"
  />
);
```

### Isoform Type

```tsx
import React from 'react';
import { GTexTranscriptViewer } from 'gtex-d3';

export default () => (
  <GTexTranscriptViewer
    rootId="gtex-transcript-viewer-isoform"
    title="GTex Transcript Viewer"
    geneId="ENSG00000141510"
    type="exon"
  />
);
```

### Junction Type

```tsx
import React from 'react';
import { GTexTranscriptViewer } from 'gtex-d3';

export default () => (
  <GTexTranscriptViewer
    rootId="gtex-transcript-viewer-junction"
    title="GTex Transcript Viewer"
    geneId="ENSG00000141510"
    type="junction"
  />
);
```

<API></API>
