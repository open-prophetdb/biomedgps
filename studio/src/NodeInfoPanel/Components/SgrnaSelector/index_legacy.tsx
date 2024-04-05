import React, { useEffect, useState } from 'react'
import { GuideScoperViewer } from 'biominer-components';

type SgrnaSelectorProps = {
  rootId?: string,
  // Only support entrezId for now
  geneId: string,
  // Only support 9606(human), 10090(mouse) for now
  taxId?: number
  url?: string
}

const formatEntrezId = (geneId: string) => {
  if (geneId.includes("ENTREZ:")) {
    return geneId;
  }

  if (geneId.match(/^\d+$/)) {
    return `ENTREZ:${geneId}`;
  }

  return geneId;
}

const SgrnaSelector: React.FC<SgrnaSelectorProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");

  useEffect(() => {
    if (!props.rootId) {
      setRootId('grna-selector')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <GuideScoperViewer geneId={formatEntrezId(props.geneId)} id={rootId}
      url={"https://biosolver.cn/#/grna-query-details?entrezId="}>
    </GuideScoperViewer>
  )
}

export default SgrnaSelector;