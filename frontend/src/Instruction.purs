module Instruction (
  Instruction(..),
  Behaviour(..)
) where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Argonaut.Decode (decodeJson, (.:))
import Data.Argonaut.Decode.Class (class DecodeJson)
import Data.Argonaut.Decode.Error (JsonDecodeError(..))
import Data.Argonaut.Encode (encodeJson)
import Data.Argonaut.Encode.Class (class EncodeJson)
import Data.Maybe (Maybe)

data Instruction
  = Idle
  | Play String (Maybe Number) Behaviour

instance DecodeJson Instruction where
  decodeJson j = do
    o <- decodeJson j
    tag <- o .: "tag"
    case tag of
      "Idle" ->
        pure Idle
      "Play" ->
        Play <$> o .: "path" <*> o .: "from" <*> o .: "behaviour"
      _ ->
        throwError (UnexpectedValue $ encodeJson tag)

instance EncodeJson Instruction where
  encodeJson = case _ of
    Idle ->
      encodeJson { tag: "Idle" }
    Play path from behaviour ->
      encodeJson { tag: "Play", path, from, behaviour }

data Behaviour
  = Playing
  | Paused

instance DecodeJson Behaviour where
  decodeJson j = do
    o <- decodeJson j
    tag <- o .: "tag"
    case tag of
      "Playing" ->
        pure Playing
      "Paused" ->
        pure Paused
      _ ->
        throwError (UnexpectedValue $ encodeJson tag)

instance EncodeJson Behaviour where
  encodeJson = case _ of
    Playing ->
      encodeJson { tag: "Playing" }
    Paused ->
      encodeJson { tag: "Paused" }
