import LanguageIcon from "@mui/icons-material/Language";
import assertNever from "assert-never";
import { useRecoilState } from "recoil";
import MyIconButton from "../../components/MyIconButton";
import { MyMenu } from "../../components/MyMenu";
import { gameState } from "../../state/gameMode";

export function WorldSettingsButton() {
    const [gameMode, setGameMode] = useRecoilState(gameState);

    return (
        <MyMenu
            menuItems={[
                {
                    label: `Toggle game mode (${gameMode.mode})`,
                    // icon: <OpenInNewIcon />,
                    onClick: () => {
                        setGameMode((gameMode) => {
                            if (gameMode.mode === "sudoku") {
                                return {
                                    mode: "world",
                                    view: "sudoku",
                                    selectedGridIndex: { row: 0, column: 0 },
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
                              onClick: () => {
                                  setGameMode({
                                      ...gameMode,
                                      view: gameMode.view === "sudoku" ? "map" : "sudoku",
                                  });
                              },
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
