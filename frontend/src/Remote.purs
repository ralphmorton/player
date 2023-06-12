module Remote where

import Prelude hiding (div)

import Control.Monad.Rec.Class (forever)
import Data.Argonaut.Encode (toJsonString)
import Data.HTTP.Method (Method(..))
import Data.Maybe (Maybe(..))
import Data.Newtype (wrap)
import Data.Tuple.Nested ((/\))
import Effect (Effect)
import Effect.Aff (Aff, delay, launchAff_)
import Effect.Class (liftEffect)
import Fetch (fetch)
import Fetch.Argonaut.Json (fromJson)
import Instruction (Instruction(..))
import Muon (Html, Muon, Signal, a, click, div, i, muon, on, state, text, (:=))

type State = {
  files :: Array String,
  selected :: Maybe String
}

type StateChans = {
  files :: Array String -> Effect Unit,
  selected :: Maybe String -> Effect Unit
}

main :: Effect Unit
main = muon =<< app

app :: Effect (Signal (Muon Html))
app = do
  sig /\ chans <- state { files: [], selected: Nothing }
  list chans
  pure $ sig <#> \{ files, selected } -> pure $
    div ["class" := "container"] $
      case selected of
        Just path ->
          [
            div ["class" := "title"] [text path],
            div ["class" := "controls"] [
              a ["href" := "#", on click (const resume)] [
                i ["class" := "bx bx-play"] []
              ],
              a ["href" := "#", on click (const pause)] [
                i ["class" := "bx bx-pause"] []
              ],
              a ["href" := "#", on click (const $ stop chans)] [
                i ["class" := "bx bx-stop"] []
              ]
            ]
          ]
        Nothing ->
          files <#> \file ->
            div ["class" := "entry"] [
              a ["href" := "#", on click (const $ play chans file)] [text file]
            ]

ifHtml :: Boolean -> Html -> Html
ifHtml c h = if c then h else text ""

--
-- Network stuff
--

list :: StateChans -> Effect Unit
list chans = launchAff_ $ forever do
  req chans
  delay (wrap 5000.0)

req :: StateChans -> Aff Unit
req chans = do
  { json } <- fetch "/ls" {
    method: GET,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  files <- fromJson json
  liftEffect $ chans.files files

play :: StateChans -> String -> Effect Unit
play chans path = do
  instruction (Play path)
  chans.selected (pure path)

resume :: Effect Unit
resume = instruction Resume

pause :: Effect Unit
pause = instruction Pause

stop :: StateChans -> Effect Unit
stop chans = do
  instruction Stop
  chans.selected Nothing

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
