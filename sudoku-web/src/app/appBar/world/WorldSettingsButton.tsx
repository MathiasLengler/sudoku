import LanguageIcon from "@mui/icons-material/Language";
import assertNever from "assert-never";
import { useRecoilState } from "recoil";
import MyIconButton from "../../components/MyIconButton";
import { MyMenu } from "../../components/MyMenu";
import { allWorldCellsState, gameModeState } from "../../state/world";
import { useChangeTile } from "../../actions/worldActions";

export function WorldSettingsButton() {
    const [gameMode, setGameMode] = useRecoilState(gameModeState);
    const changeTile = useChangeTile();

    return (
        <MyMenu
            menuItems={[
                {
                    label: `Toggle game mode (${gameMode.mode})`,
                    // icon: <OpenInNewIcon />,
                    onClick: () => {
                        setGameMode(gameMode => {
                            if (gameMode.mode === "sudoku") {
                                return {
                                    mode: "world",
                                    view: "sudoku",
                                    currentTileIndex: { row: 0, column: 0 },
                                };
                            } else if (gameMode.mode === "world") {
                                return {
                                    mode: "sudoku",
                                };
                            } else {
                                assertNever(gameMode);
                            }
                        });
                    },
                },
                ...(gameMode.mode === "world"
                    ? [
                          {
                              label: `Toggle world view (${gameMode.view})`,
                              onClick: async () => {
                                  setGameMode({
                                      ...gameMode,
                                      view: gameMode.view === "sudoku" ? "map" : "sudoku",
                                  });
                              },
                          },
                          {
                              label: "Tile ←",
                              onClick: async () => await changeTile("left"),
                          },
                          {
                              label: "Tile →",
                              onClick: async () => await changeTile("right"),
                          },
                          {
                              label: "Tile ↑",
                              onClick: async () => await changeTile("top"),
                          },
                          {
                              label: "Tile ↓",
                              onClick: async () => await changeTile("bottom"),
                          },
                      ]
                    : []),
            ]}
        >
            {({ onMenuOpen }) => (
                <MyIconButton
                    label="World settings"
                    icon={LanguageIcon}
                    size="large"
                    color="inherit"
                    onClick={onMenuOpen}
                />
            )}
        </MyMenu>
    );
}
