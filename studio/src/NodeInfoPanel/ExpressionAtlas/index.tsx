import React, { useEffect, useState } from 'react';
import { Spin } from 'antd';

type ExpressionAtlasProps = {
  rootId?: string,
  // Only support official gene symbol for now
  geneSymbol: string,
  // Only support 9606(human), 10090(mouse) for now
  taxId?: number
}

const ExpressionAtlas: React.FC<ExpressionAtlasProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [src, setSrc] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    if (props.geneSymbol) {
      setSrc(`https://omics-data.3steps.cn/fetch/expression_atlas?geneSymbol=${props.geneSymbol}&taxId=${props.taxId || 9606}`)
    }
  }, [props.geneSymbol]);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('expression_atlas')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <div id="iframe-container" style={{ position: 'relative', width: '100%', height: '100%' }}>
      <iframe id={rootId} title="Expression Atlas" src={src} onLoad={() => setLoading(false)}
        style={{ width: '100%', height: '100%', border: 'none', minHeight: '1000px' }} />
      {
        loading ? <Spin spinning={loading} style={{
          position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', minHeight: '1000px'
        }}></Spin> : null
      }
    </div>
  )
}

export default ExpressionAtlas;