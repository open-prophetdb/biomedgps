import React, { useEffect, useState, useRef } from 'react'
import { Spin } from 'antd'

type SangerCosmicProps = {
  rootId?: string,
  // Only support official gene symbol for now
  geneSymbol: string,
  // Only support 9606(human), 10090(mouse) for now
  taxId?: number
}

const SangerCosmic: React.FC<SangerCosmicProps> = (props) => {
  const [rootId, setRootId] = useState<string>("");
  const [src, setSrc] = useState<string>("");
  const ref = useRef(null);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    if (props.geneSymbol) {
      let host = window.location.host;
      if (host.startsWith('localhost')) {
        host = "drugs.3steps.cn"
      }

      setSrc(`https://${host}/proxy/sanger_cosmic/cosmic/gene/analysis?ln=${props.geneSymbol}&taxid=${props.taxId || 9606}`)
    }
  }, [props.geneSymbol]);

  useEffect(() => {
    if (!props.rootId) {
      setRootId('sanger-cosmic')
    } else {
      setRootId(props.rootId)
    }
  }, []);

  return (
    <div className="iframe-container" style={{ position: 'relative', width: '100%', height: '100%' }}>
      <iframe id={rootId} title="Cosmic Mutations" src={src}
        style={{ width: '100%', height: '100%', border: 'none', minHeight: '1000px' }} onLoad={() => {
          setLoading(false)
        }} ref={ref} />
      {
        loading ? <Spin spinning={loading} style={{
          position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', minHeight: '1000px'
        }}></Spin> : null
      }
    </div>
  )
}

export default SangerCosmic;