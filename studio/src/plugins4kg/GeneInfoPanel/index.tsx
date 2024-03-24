import React, { useEffect, useState } from 'react'

type GeneInfoProps = {
  rootId?: string,
  // Only support official gene symbol for now
  geneSymbol: string,
  // Only support 9606(human), 10090(mouse) for now
  taxId?: number
}

const GeneInfo: React.FC<GeneInfoProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [src, setSrc] = useState<string>("");

  useEffect(() => {
    if (props.geneSymbol) {
      setSrc(`https://omics-data.3steps.cn/fetch/gene_cards?geneSymbol=${props.geneSymbol}&taxId=${props.taxId || 9606}`)
    }
  }, [props.geneSymbol]);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('gene-info')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <iframe id={rootId} title="Gene Information" src={src}
      style={{ width: '100%', height: '100%', border: 'none', minHeight: '1000px' }} />
  )
}

export default GeneInfo;