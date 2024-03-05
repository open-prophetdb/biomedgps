import Molstar from "molstar-react";

export const MolStarViewer = () => {

  return (
    <div>
      <img width={'100%'} src="/examples/molstar.png" />
      {/* <Molstar url={"https://alphafold.ebi.ac.uk/files/AF-E9PLR3-F1-model_v4.pdb"} /> */}
    </div>
  );
}

export default MolStarViewer;