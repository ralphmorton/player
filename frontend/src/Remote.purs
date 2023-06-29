module Remote where

import Prelude hiding (div)

import Control.Monad.Rec.Class (forever)
import Data.Argonaut.Encode (toJsonString)
import Data.Array (sort)
import Data.HTTP.Method (Method(..))
import Data.Int (floor)
import Data.Maybe (Maybe(..))
import Data.Newtype (wrap)
import Data.Tuple.Nested ((/\))
import Effect (Effect)
import Effect.Aff (Aff, delay, launchAff_)
import Effect.Class (liftEffect)
import Fetch (fetch)
import Fetch.Argonaut.Json (fromJson)
import Instruction (Behaviour(..), Instruction(..))
import Muon (Html, Muon, Signal, a, click, div, i, muon, on, state, text, (:=))
import PlayerState (PlayerState)

type State = {
  files :: Array String,
  playerState :: Maybe PlayerState
}

type StateChans = {
  files :: Array String -> Effect Unit,
  playerState :: Maybe PlayerState -> Effect Unit
}

main :: Effect Unit
main = muon =<< app

app :: Effect (Signal (Muon Html))
app = do
  sig /\ chans <- state { files: [], playerState: Nothing }
  list chans
  poll chans
  pure $ sig <#> \{ files, playerState } -> pure $
    div ["class" := "container pt-5"] [
      div ["class" := "row"] [
        div ["class" := "col-12"] $
          case playerState of
            Just { path, duration, time } ->
              [
                div ["class" := "card"] [
                  div ["class" := "card-body text-center"] [
                    div ["class" := "h3 m-0 text-warning"] [text path],
                    div ["class" := "h3 my-4"] [
                      a ["class" := "mr-4", "href" := "#", on click (const $ play path $ pure $ max 0.0 (time - 60.0))] [
                        i ["class" := "bx bx-chevrons-left"] []
                      ],
                      a ["class" := "mr-4", "href" := "#", on click (const $ resume path)] [
                        i ["class" := "bx bx-play"] []
                      ],
                      a ["class" := "mr-4", "href" := "#", on click (const $ pause path)] [
                        i ["class" := "bx bx-pause"] []
                      ],
                      a ["class" := "mr-4", "href" := "#", on click (const stop)] [
                        i ["class" := "bx bx-stop"] []
                      ],
                      a ["class" := "mr-4", "href" := "#", on click (const $ play path $ pure $ min duration (time + 60.0))] [
                        i ["class" := "bx bx-chevrons-right"] []
                      ]
                    ],
                    div ["class" := "progress"] [
                      div
                        [
                          "class" := "progress-bar",
                          "role" := "progressbar",
                          "style" := ("width: " <> show (floor $ (time / duration) * 100.0) <> "%;")
                        ]
                        []
                    ]
                  ]
                ]
              ]
            Nothing ->
              [
                div ["class" := "card"] [
                  div ["class" := "card-body"] $
                    sort files <#> \file ->
                      div ["class" := "mb-3 p-3 border"] [
                        a ["href" := "#", on click (const $ play file Nothing)] [text file]
                      ]
                ]
              ]
      ]
    ]

ifHtml :: Boolean -> Html -> Html
ifHtml c h = if c then h else text ""

--
-- Network stuff
--

list :: StateChans -> Effect Unit
list chans = launchAff_ $ forever do
  listFiles chans
  delay (wrap 5000.0)

listFiles :: StateChans -> Aff Unit
listFiles chans = do
  { json } <- fetch "/ls" {
    method: GET,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  files <- fromJson json
  liftEffect $ chans.files files

poll :: StateChans -> Effect Unit
poll chans = launchAff_ $ forever do
  pollPlayerState chans
  delay (wrap 250.0)

pollPlayerState :: StateChans -> Aff Unit
pollPlayerState chans = do
  { json } <- fetch "/state" {
    method: GET,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  liftEffect <<< chans.playerState =<< fromJson json

play :: String -> Maybe Number -> Effect Unit
play path from = instruction (Play path from Playing)

resume :: String -> Effect Unit
resume path = instruction (Play path Nothing Playing)

pause :: String -> Effect Unit
pause path = instruction (Play path Nothing Paused)

stop :: Effect Unit
stop = instruction Idle

instruction :: Instruction -> Effect Unit
instruction i = launchAff_ do
  void $ fetch "/instruction" {
    method: POST,
    body: toJsonString i,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
