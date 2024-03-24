import React, { useEffect, useRef, useState } from "react";
import { DefaultPluginSpec } from "molstar/lib/mol-plugin/spec";
import { DefaultPluginUISpec } from "molstar/lib/mol-plugin-ui/spec";
import { createPluginUI } from "molstar/lib/mol-plugin-ui/index";
import { PluginContext } from "molstar/lib/mol-plugin/context";
import "molstar/build/viewer/molstar.css";
import { ParamDefinition } from "molstar/lib/mol-util/param-definition";
import { CameraHelperParams } from "molstar/lib/mol-canvas3d/helper/camera-helper";

type MolstarViewerProps = {
  useInterface?: boolean;
  pdbId?: string;
  url?: string;
  file?: any;
  dimensions?: [string, string];
  showControls?: boolean;
  showAxes?: boolean;
  className?: string;
};

const MolstarViewer: React.FC<MolstarViewerProps> = (props) => {
  const { useInterface, pdbId, url, file, dimensions, className, showControls, showAxes } = props;
  const parentRef = useRef(null);
  const canvasRef = useRef(null);
  const plugin = useRef(null);
  const [initialized, setInitialized] = useState(false);

  useEffect(() => {
    const init = async () => {
      if (useInterface) {
        const spec = DefaultPluginUISpec();
        spec.layout = {
          initial: {
            isExpanded: false,
            controlsDisplay: "reactive",
            showControls,
          }
        };

        if (parentRef.current) {
          // @ts-ignore
          plugin.current = await createPluginUI(parentRef.current, spec);
        }
      } else {
        // @ts-ignore
        plugin.current = new PluginContext(DefaultPluginSpec());
        // @ts-ignore
        plugin.current.initViewer(canvasRef.current, parentRef.current);
        // @ts-ignore
        await plugin.current.init();
      }

      if (!showAxes) {
        // @ts-ignore
        plugin.current.canvas3d?.setProps({
          camera: {
            helper: {
              axes: {
                name: "off", params: {}
              }
            }
          }
        });
      }

      // @ts-ignore
      await loadStructure(pdbId, url, file, plugin.current);
      setInitialized(true);
    }

    init();

    return () => {
      plugin.current = null;
    }
  }, [])


  useEffect(() => {
    if (!initialized) return;
    (async () => {
      // @ts-ignore
      await loadStructure(pdbId, url, file, plugin.current);
    })();
  }, [pdbId, url, file])


  useEffect(() => {
    if (plugin.current) {
      if (!showAxes) {
        // @ts-ignore
        plugin.current.canvas3d?.setProps({
          camera: {
            helper: {
              axes: {
                name: "off", params: {}
              }
            }
          }
        })
      } else {
        // @ts-ignore
        plugin.current.canvas3d?.setProps({
          camera: {
            helper: {
              axes: ParamDefinition.getDefaultValues(CameraHelperParams).axes
            }
          }
        })
      }
    }
  }, [showAxes])


  const loadStructure = async (pdbId: string, url: string, file: any, plugin: any) => {
    if (plugin) {
      plugin.clear();
      if (file) {
        const data = await plugin.builders.data.rawData({
          data: file.filestring
        });
        const traj = await plugin.builders.structure.parseTrajectory(data, file.type);
        await plugin.builders.structure.hierarchy.applyPreset(traj, "default");
      } else {
        const structureUrl = url ? url : pdbId ? `https://files.rcsb.org/view/${pdbId}.cif` : null;
        if (!structureUrl) return;
        const data = await plugin.builders.data.download(
          { url: structureUrl }, { state: { isGhost: true } }
        );

        // @ts-ignore
        let extension = structureUrl.split(".").pop().replace("cif", "mmcif");
        if (extension.includes("?"))
          extension = extension.substring(0, extension.indexOf("?"));
        const traj = await plugin.builders.structure.parseTrajectory(data, extension);
        await plugin.builders.structure.hierarchy.applyPreset(traj, "default");
      }
    }
  }

  const width = dimensions ? dimensions[0] : "100%";
  const height = dimensions ? dimensions[1] : "100%";

  if (useInterface) {
    return (
      <div style={{ position: "relative", width, height, overflow: "hidden" }}>
        <div ref={parentRef} style={{ position: "relative", left: 0, top: 0, right: 0, bottom: 0, height: '100%' }} />
      </div>
    )
  }

  return (
    <div
      ref={parentRef}
      style={{ position: "relative", width, height }}
      className={className || ""}
    >
      <canvas
        ref={canvasRef}
        style={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0 }}
      />
    </div>
  );
};

export default MolstarViewer;
