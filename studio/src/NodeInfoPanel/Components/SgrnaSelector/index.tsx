import React, { useEffect, useState, useRef } from 'react';
import { Spin } from 'antd';

type GuideScoperViewerProps = {
    id?: string,
    // Only support entrezId for now
    geneId: string,
    // Only support 9606(human), 10090(mouse) for now
    taxId?: number,
    url?: string,
    width?: number,
    height?: number
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

const genLink = (url: string, geneId: string, taxid?: number) => {
    if (taxid == 9606 || taxid == 10090) {
        return `${url}${geneId}&taxid=${taxid}&isEmbeded=true`;
    } else {
        return `${url}${geneId}&isEmbeded=true`;
    }
};

const defaultUrl = 'https://biosolver.cn/#/grna-query-details?entrezId=';

const GuideScoperViewer: React.FC<GuideScoperViewerProps> = (props) => {
    const { geneId, taxId, url, id } = props;
    const ref = useRef(null);
    const [rootId, setRootId] = useState<string>(id || 'guide-scoper-viewer');
    const [src, setSrc] = useState<string>(genLink(defaultUrl, geneId, taxId));
    const [loading, setLoading] = useState<boolean>(true);

    useEffect(() => {
        if (url && url !== defaultUrl) {
            const newSrc = genLink(url, geneId, taxId);
            setSrc(newSrc);

            window.addEventListener('message', (event) => {
                const defaultBaseUrl = new URL(src).origin;
                if (event.origin === defaultBaseUrl) {
                    if (event.data && event.data.type === 'resizeIframe') {
                        const iframe = document.getElementById(rootId);
                        if (iframe) {
                            if (!props.height) {
                                iframe.style.height = `${event.data.height}px`;
                            }
                            if (!props.width) {
                                iframe.style.width = `${event.data.width}px`;
                            }
                        }
                    }
                }
            });
        }
    }, [url]);

    useEffect(() => {
        if (id) {
            setRootId(id);
        }
    }, [id]);

    return (
        <div className="iframe-container" style={{ position: 'relative', width: '100%', height: '100%' }}>
            <iframe id={rootId} title="Guide Scoper" src={src}
                onLoad={() => setLoading(false)} ref={ref}
                className='guide-scoper-viewer'
                style={{ width: '100%', height: '100%', border: 'none', minHeight: '1000px' }} />
            {
                loading ? <Spin spinning={loading} style={{
                    position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', minHeight: '1000px'
                }}></Spin> : null
            }
        </div>
    )
}

export default GuideScoperViewer;